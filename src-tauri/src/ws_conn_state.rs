use crate::cccb_error::CccbError;
use crate::queues::ToTcp;
use futures::stream::SplitStream;
use futures_util::{StreamExt, TryStreamExt};
use std::sync::Arc;
use tauri::State;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tokio_util::sync::CancellationToken;
use tokio_util::time::delay_queue::DelayQueue;
//use tokio_websockets::ClientBuilder;
//use tokio_websockets::{tls::MaybeTlsStream, WebSocketStream};

pub type CccbWSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tauri::command]
pub async fn ws_connect(
    token: String,
    state: State<'_, WsConnState>,
    to_tcp: State<'_, ToTcp>,
) -> Result<(), String> {
    let mut ws_cancel = state.cancel.lock().await;
    if !ws_cancel.is_none() {
        return Err("already connecting/connected".into());
    }
    println!("foo: {}", token);
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

    println!("connected");

    let (_, read) = conn.split();

    println!("start polling");

    let poll_cancel = canceller.clone();

    let tcp_queue = to_tcp.queue.clone();
    // start a polling task
    //poll_incoming(&to_tcp, read).await;
    tokio::spawn(async move {
        tokio::select! {
            _ = poll_cancel.cancelled() => {
                Err("cancelled".to_string())
            }
            res = poll_incoming(tcp_queue, read) => {
                res
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
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
    println!("starting connection");
    let uri = url::Url::parse(&format!("ws://127.0.0.1:3000?token={token}"))
        .map_err(|e| e.to_string())?;

    let (client, _) = connect_async(uri).await.map_err(|e| e.to_string())?;
    Ok(client)
}

async fn poll_incoming(
    tcp_queue: Arc<Mutex<DelayQueue<String>>>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<(), String> {
    let queue_ref = &tcp_queue;
    read.try_for_each(|msg| async move {
        let queue_copy = queue_ref.clone();
        let mut queue_lock = queue_copy.lock().await;
        queue_lock.insert(msg.to_string(), Duration::new(0, 0));
        println!("{:?}", msg);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())
}

/*
async fn connect(token: String) -> Result<CccbWSStream, String> {
    let uri = format!("ws://127.0.0.1:3000?token={token}")
        .parse::<Uri>()
        .map_err(|e| e.to_string())?;

    let (client, _) = ClientBuilder::from_uri(uri)
        .connect()
        .await
        .map_err(|e| e.to_string())?;

    Ok(client)
}
*/

#[derive(Default)]
pub struct WsConnState {
    cancel: Mutex<Option<CancellationToken>>,
}
