// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod mod_statuses;
mod queues;
mod tcp_conn_state;
mod ws_conn_state;

use queues::{RetryQueue, ToTcp, ToWs};
use tcp_conn_state::TcpConnState;
use ws_conn_state::WsConnState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .manage(WsConnState::default())
        .manage(ToTcp::new())
        .manage(TcpConnState::default())
        .manage(ToWs::new())
        .manage(RetryQueue::new())
        .invoke_handler(tauri::generate_handler![
            ws_conn_state::ws_connect,
            ws_conn_state::ws_disconnect,
            tcp_conn_state::tcp_connect,
            tcp_conn_state::tcp_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running crowd control celeste bridge");

    Ok(())
}
