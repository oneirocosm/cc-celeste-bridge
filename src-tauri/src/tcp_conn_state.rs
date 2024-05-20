use crate::mod_statuses::{
    EffectResult, FromServer, RequestType, Response, ResponseType, ToServer,
};
use crate::queues::{RetryQueue, ToTcp, ToWs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::{tcp::OwnedReadHalf, tcp::OwnedWriteHalf, TcpListener};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn tcp_connect(
    state: State<'_, TcpConnState>,
    to_tcp: State<'_, ToTcp>,
    to_ws: State<'_, ToWs>,
    repeats: State<'_, RetryQueue>,
    app: AppHandle,
) -> Result<(), String> {
    let out = tcp_connect_inner(state.clone(), to_tcp, to_ws, repeats, app).await;

    // backup cancel in case something is missed somehow
    let mut tcp_cancel = state.cancel.lock().await;
    if let Some(canceller) = tcp_cancel.clone() {
        canceller.cancel();
    }
    *tcp_cancel = None;
    out
}

async fn tcp_connect_inner(
    state: State<'_, TcpConnState>,
    to_tcp: State<'_, ToTcp>,
    to_ws: State<'_, ToWs>,
    repeats: State<'_, RetryQueue>,
    app: AppHandle,
) -> Result<(), String> {
    let mut ws_cancel = state.cancel.lock().await;
    if !ws_cancel.is_none() {
        return Err("already connecting/connected".into());
    }
    let canceller = CancellationToken::new();
    let conn_cancel = canceller.clone();
    let write_cancel = canceller.clone();
    let poll_cancel = canceller.clone();
    *ws_cancel = Some(canceller);
    std::mem::drop(ws_cancel);

    let conn = tokio::spawn(async move {
        tokio::select! {
            _ = conn_cancel.cancelled() => {
                Err("cancelled".to_string())
            }
            conn = connect() => {
                conn
            }
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    app.emit_all("tcp_conn", "connected")
        .map_err(|e| e.to_string())?;

    let (read_stream, write_stream) = conn.into_split();
    let requests: Arc<Mutex<HashMap<u32, Request>>> = Arc::new(Mutex::new(HashMap::new()));

    let tcp_rx = to_tcp.rx.clone();
    let write_requests = requests.clone();

    let writer = tokio::spawn(write_to_tcp(
        tcp_rx,
        write_stream,
        write_requests,
        write_cancel,
    ));

    let ws_tx = to_ws.tx.clone();
    let read_requests = requests.clone();
    let repeat_queue = repeats.repeats.clone();
    let tcp_tx = to_tcp.tx.clone();

    let poller = tokio::spawn(poll_from_tcp(
        read_stream,
        ws_tx,
        read_requests,
        repeat_queue,
        tcp_tx,
        poll_cancel,
    ));

    let (write_out, read_out) = tokio::try_join!(writer, poller).map_err(|e| e.to_string())?;
    match (write_out, read_out) {
        (Err(write_err), Err(read_err)) => Err(format!("{}, {}", write_err, read_err)),
        (Err(write_err), Ok(_)) => Err(write_err),
        (Ok(_), Err(read_err)) => Err(read_err),
        (Ok(_), Ok(_)) => Ok(()),
    }
}

#[tauri::command]
pub async fn tcp_disconnect(state: State<'_, TcpConnState>) -> Result<(), String> {
    let mut tcp_cancel = state.cancel.lock().await;
    if let Some(canceller) = tcp_cancel.clone() {
        canceller.cancel();
    }
    *tcp_cancel = None;
    Ok(())
}

async fn connect() -> Result<TcpStream, String> {
    let listener = TcpListener::bind("127.0.0.1:58430")
        .await
        .map_err(|e| e.to_string())?;
    let (socket, _) = listener.accept().await.expect("oops");

    Ok(socket)
}

async fn poll_from_tcp(
    read_stream: OwnedReadHalf,
    ws_tx: Arc<Mutex<Sender<ToServer>>>,
    requests: Arc<Mutex<HashMap<u32, Request>>>,
    repeats: Arc<Mutex<VecDeque<Response>>>,
    tcp_tx: Arc<Mutex<Sender<String>>>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let mut repeat_queue = repeats.lock().await;
    loop {
        if cancel.is_cancelled() {
            return Ok(());
        }
        let mut buffer = vec![];
        let result = read_stream.try_read_buf(&mut buffer);
        if result.is_err() {
            continue;
        }
        let line = String::from_utf8_lossy(&buffer);

        let resp: Response;
        let deserialize = serde_json::from_str(line.trim_matches('\0')).map_err(|e| e.to_string());
        if let Ok(res_ok) = deserialize {
            resp = res_ok
        } else {
            continue;
        }

        match (resp.status, resp.t) {
            (EffectResult::Retry, _) => {
                repeat_queue.push_front(resp);
            }
            (_, ResponseType::EffectRequest) => {
                let mut req_lock = requests.lock().await;
                let this_req: Request;
                if let Some(res_ok) = req_lock.get(&resp.id) {
                    this_req = res_ok.clone();
                } else {
                    continue;
                }
                req_lock.remove(&resp.id);
                std::mem::drop(req_lock);

                let to_server = ToServer {
                    player_id: this_req.targets.unwrap()[0].id.clone(),
                    code: this_req.code.unwrap(),
                    time: resp.time_remaining,
                    sender: "".to_string(),
                };
                let tx_lock = ws_tx.lock().await;
                tx_lock.send(to_server).map_err(|e| e.to_string())?;
            }
            (_, _) => {
                let response: Response;
                if let Some(inner_response) = repeat_queue.pop_back() {
                    response = inner_response;
                } else {
                    continue;
                }
                let mut req_lock = requests.lock().await;
                let this_req: Request;
                if let Some(res_ok) = req_lock.get(&response.id) {
                    this_req = res_ok.clone();
                } else {
                    continue;
                }
                req_lock.remove(&resp.id);
                std::mem::drop(req_lock);

                let from_server = FromServer {
                    player_id: this_req.targets.unwrap()[0].id.clone(),
                    code: this_req.code.unwrap(),
                };

                if let Ok(retry) = serde_json::to_string(&from_server) {
                    let tx_lock = tcp_tx.lock().await;
                    tx_lock.send(retry).map_err(|e| e.to_string())?;
                }
            }
        }
    }
}

async fn write_to_tcp(
    tcp_rx: Arc<Mutex<Receiver<String>>>,
    mut write_stream: OwnedWriteHalf,
    requests: Arc<Mutex<HashMap<u32, Request>>>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let mut i = 0;
    let rx = tcp_rx.lock().await;
    loop {
        if cancel.is_cancelled() {
            return Ok(());
        }
        let msg: String;
        if let Ok(res_ok) = rx.try_recv() {
            msg = res_ok;
        } else {
            continue;
        }
        let info: FromServer = serde_json::from_str(&msg).map_err(|e| e.to_string())?;

        let target = Target {
            id: info.player_id,
            name: "".to_string(),
            avatar: "".to_string(),
        };

        let to_game = Request {
            id: i,
            code: Some(info.code),
            message: None,
            parameters: Some(Vec::new()),
            targets: Some(vec![target]),
            viewer: None,
            cost: None,
            r#type: RequestType::Start,
        };

        let mut req_lock = requests.lock().await;
        req_lock.insert(i, to_game.clone());
        std::mem::drop(req_lock);

        let req = serde_json::to_string(&to_game).map_err(|e| e.to_string())? + "\0";
        write_stream
            .write_all(req.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        write_stream.flush().await.map_err(|e| e.to_string())?;

        i += 1;
    }
}

#[derive(Default)]
pub struct TcpConnState {
    cancel: Mutex<Option<CancellationToken>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Target {
    id: String,
    name: String,
    avatar: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Request {
    id: u32,
    code: Option<String>,
    message: Option<String>,
    // the string in parameters is a placeholder
    parameters: Option<Vec<String>>,
    targets: Option<Vec<Target>>,
    viewer: Option<String>,
    cost: Option<u64>,
    r#type: RequestType,
}
