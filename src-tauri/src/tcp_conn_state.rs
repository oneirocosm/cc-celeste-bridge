use crate::queues::ToTcp;
use std::sync::Arc;
use tauri::State;
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

    let (read_stream, mut write_stream) = conn.into_split();

    println!("connected");
    Ok(())
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

async fn write_to_tcp(
    tcp_queue: Arc<Mutex<DelayQueue<String>>>,
    write_stream: OwnedWriteHalf,
) -> Result<(), String> {
    loop {
        let queue = tcp_queue.lock().await;
    }
    Ok(())
}

#[derive(Default)]
pub struct TcpConnState {
    cancel: Mutex<Option<CancellationToken>>,
}
