// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod cccb_error;
mod mod_statuses;
mod mod_tcp_conn;
mod overlay_ws_conn;
mod queues;
mod tcp_conn_state;
mod ws_conn_state;

use queues::ToTcp;
use tcp_conn_state::TcpConnState;
use ws_conn_state::WsConnState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        /*
        .manage(AppState {
            ws_conn: Default::default(),
        })
        */
        .manage(WsConnState::default())
        .manage(ToTcp::default())
        .manage(TcpConnState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            mod_tcp_conn::connect,
            mod_tcp_conn::listen,
            overlay_ws_conn::connect_overlay,
            ws_conn_state::ws_connect,
            ws_conn_state::ws_disconnect,
            tcp_conn_state::tcp_connect,
            tcp_conn_state::tcp_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running crowd control celeste bridge");

    Ok(())
}
