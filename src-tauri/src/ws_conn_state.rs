use crate::cccb_error::CccbError;
use http::Uri;
use std::sync::Arc;
use tauri::State;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tokio_websockets::ClientBuilder;
use tokio_websockets::{tls::MaybeTlsStream, WebSocketStream};

pub type CccbWSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tauri::command]
pub async fn ws_connect(token: String, state: State<'_, WsConnState>) -> Result<(), String> {
    let mut connection = state.connection.lock().await;
    let mut conn_cancel = state.conn_cancel.lock().await;
    if !connection.is_none() || !conn_cancel.is_none() {
        return Ok(());
    }
    if connection.is_none() {
        return Err("already connecting".into());
    }
    let canceller = CancellationToken::new();
    *conn_cancel = Some(canceller.clone());
    std::mem::drop(conn_cancel);

    let conn = tokio::spawn(async move {
        tokio::select! {
            _ = canceller.cancelled() => {
                Err("cancelled".to_string())
            }
            conn = connect(token) => {
                conn
            }
        }
    })
    .await
    .map_err(|e| e.to_string());

    // we no longer need a canceller
    let mut conn_cancel = state.conn_cancel.lock().await;
    *conn_cancel = None;
    *connection = Some(conn??);

    // start a polling task
    let mut poll_cancel = state.poll_cancel.lock().await;
    //tokio::spawn();

    Ok(())
}

#[tauri::command]
pub async fn ws_disconnect(state: State<'_, WsConnState>) -> Result<(), String> {
    let conn_cancel = state.conn_cancel.lock().await;
    if let Some(canceller) = conn_cancel.clone() {
        canceller.cancel();
    }
    std::mem::drop(conn_cancel);

    let mut connection = state.connection.lock().await;
    *connection = None;
    Ok(())
}

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

async fn poll(state: WsConnState) -> Result<(), String> {
    let connection = state.connection.lock().await;
    //if let Some(conn) = connection.clone() {}

    Ok(())
}

#[derive(Default)]
pub struct WsConnState {
    connection: Mutex<Option<CccbWSStream>>,
    conn_cancel: Mutex<Option<CancellationToken>>,
    poll_cancel: Mutex<Option<CancellationToken>>,
}
