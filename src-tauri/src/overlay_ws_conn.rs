use futures_util::StreamExt;
use http::Uri;
use tokio_websockets::ClientBuilder;

#[tauri::command]
pub async fn connect_overlay() -> Result<(), String> {
    let uri = Uri::from_static("ws://127.0.0.1:3000?token=QWMuZUF");
    let (mut client, _) = ClientBuilder::from_uri(uri)
        .connect()
        .await
        .map_err(|e| e.to_string())?;
    println!("connected");

    while let Some(item) = client.next().await {
        println!("{item:?}");
    }
    println!("disconnected");

    Ok(())
}
