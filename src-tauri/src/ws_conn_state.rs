use crate::mod_statuses::ToServer;
use crate::queues::{ToTcp, ToWs};
use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tokio_util::sync::CancellationToken;

pub type CccbWSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tauri::command]
pub async fn ws_connect(
    token: String,
    state: State<'_, WsConnState>,
    to_tcp: State<'_, ToTcp>,
    to_ws: State<'_, ToWs>,
    app: AppHandle,
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
            conn = connect(token) => {
                conn
            }
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    app.emit_all("ws_conn", "connected")
        .map_err(|e| e.to_string())?;

    let (write, read) = conn.split();

    let write_cancel = canceller.clone();
    let ws_rx = to_ws.rx.clone();

    let writer = tokio::spawn(write_to_ws(ws_rx, write, write_cancel));

    let poll_cancel = canceller.clone();
    let tcp_tx = to_tcp.tx.clone();

    let poller = tokio::spawn(async move {
        tokio::select! {
            _ = poll_cancel.cancelled() => {
                Err("cancelled".to_string())
            }
            res = poll_incoming(tcp_tx, read) => {
                res
            }
        }
    });

    let (write_out, read_out) = tokio::try_join!(writer, poller).map_err(|e| e.to_string())?;
    match (write_out, read_out) {
        (Err(write_err), Err(read_err)) => Err(format!("{}, {}", write_err, read_err)),
        (Err(write_err), Ok(_)) => Err(write_err),
        (Ok(_), Err(read_err)) => Err(read_err),
        (Ok(_), Ok(_)) => Ok(()),
    }
}

#[tauri::command]
pub async fn ws_disconnect(state: State<'_, WsConnState>) -> Result<(), String> {
    let mut ws_cancel = state.cancel.lock().await;
    if let Some(canceller) = ws_cancel.clone() {
        canceller.cancel();
    }
    *ws_cancel = None;
    Ok(())
}

async fn connect(token: String) -> Result<CccbWSStream, String> {
    let uri = url::Url::parse(&format!("ws://127.0.0.1:3000?token={token}"))
        .map_err(|e| e.to_string())?;

    let (client, _) = connect_async(uri).await.map_err(|e| e.to_string())?;
    Ok(client)
}

async fn poll_incoming(
    tcp_queue: Arc<Mutex<Sender<String>>>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<(), String> {
    let tx_ref = &tcp_queue;
    read.try_for_each(|msg| async move {
        let tx_copy = tx_ref.clone();
        let tx_lock = tx_copy.lock().await;
        tx_lock
            .send(msg.to_string())
            .map_err(|_e| tokio_tungstenite::tungstenite::Error::WriteBufferFull(msg.clone()))?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())
}

async fn write_to_ws(
    ws_rx: Arc<Mutex<Receiver<ToServer>>>,
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let rx = ws_rx.lock().await;
    loop {
        if cancel.is_cancelled() {
            return Ok(());
        }
        let msg: ToServer;
        if let Ok(res_ok) = rx.try_recv() {
            msg = res_ok;
        } else {
            continue;
        }
        let serialized = serde_json::to_string(&msg).map_err(|e| e.to_string())?;

        write
            .send(Message::Text(serialized))
            .await
            .map_err(|e| e.to_string())?;
    }
}

#[derive(Default)]
pub struct WsConnState {
    cancel: Mutex<Option<CancellationToken>>,
}
