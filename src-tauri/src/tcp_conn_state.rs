use crate::mod_statuses::{FromServer, RequestType};
use crate::queues::ToTcp;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tauri::State;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::{tcp::OwnedReadHalf, tcp::OwnedWriteHalf, TcpListener};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tokio_util::time::delay_queue::DelayQueue;

#[tauri::command]
pub async fn tcp_connect(
    state: State<'_, TcpConnState>,
    to_tcp: State<'_, ToTcp>,
) -> Result<(), String> {
    let mut ws_cancel = state.cancel.lock().await;
    if !ws_cancel.is_none() {
        return Err("already connecting/connected".into());
    }
    let canceller = CancellationToken::new();
    *ws_cancel = Some(canceller.clone());
    std::mem::drop(ws_cancel);
    let conn_cancel = canceller.clone();

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

    let (read_stream, write_stream) = conn.into_split();

    let tcp_rx = to_tcp.rx.clone();

    /*
    let coroutines = vec![
        check_cancel(poll_cancel),
        write_to_tcp(tcp_rx, write_stream),
        poll_from_tcp(read_stream),
    ];*/

    let write_cancel = canceller.clone();

    let writer = tokio::spawn(async move {
        tokio::select! {
            _ = write_cancel.cancelled() => {
                Err("cancelled".to_string())
            },
            res = write_to_tcp(tcp_rx, write_stream) => {
                res
            },
        }
    });

    let poll_cancel = canceller.clone();

    let poller = tokio::spawn(async move {
        tokio::select! {
            _ = poll_cancel.cancelled() => {
                Err("cancelled".to_string())
            },
            res = poll_from_tcp(read_stream) => {
                res
            },
        }
    });

    tokio::select!(
    res = writer => {
        res
    },
    res = poller => {
        res
    })
    .map_err(|e| e.to_string())?
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

async fn poll_from_tcp(read_stream: OwnedReadHalf) -> Result<(), String> {
    println!("tcp poll");
    loop {
        let mut buffer = vec![];
        let result = read_stream.try_read_buf(&mut buffer);
        if let Err(_) = result {
            continue;
        }
        let line = String::from_utf8_lossy(&buffer);

        println!("received: {}", line)
    }
}

async fn write_to_tcp(
    tcp_rx: Arc<Mutex<Receiver<String>>>,
    mut write_stream: OwnedWriteHalf,
) -> Result<(), String> {
    println!("tcp write");
    let mut i = 0;
    let rx = tcp_rx.lock().await;
    loop {
        let msg: String;
        if let Ok(res_ok) = rx.recv() {
            msg = res_ok;
        } else {
            return Err("tcp error".to_string());
        }
        println!("{}", msg);
        let info: FromServer = serde_json::from_str(&msg).expect("a");

        let to_game = Request {
            id: i,
            code: Some(info.code),
            message: None,
            parameters: Some(Vec::new()),
            targets: None,
            viewer: None,
            cost: None,
            r#type: RequestType::Start,
        };
        let req = serde_json::to_string(&to_game).expect("bar") + "\0";

        write_stream.write_all(req.as_bytes()).await.expect("aaa");
        write_stream.flush().await.expect("flush");
        i += 1;
    }
}

#[derive(Default)]
pub struct TcpConnState {
    cancel: Mutex<Option<CancellationToken>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Target {
    id: String,
    name: String,
    avatar: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    id: u64,
    code: Option<String>,
    message: Option<String>,
    // the string in parameters is a placeholder
    parameters: Option<Vec<String>>,
    targets: Option<Vec<Target>>,
    viewer: Option<String>,
    cost: Option<u64>,
    r#type: RequestType,
}
