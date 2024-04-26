use crate::mod_statuses;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{tcp::OwnedReadHalf, TcpListener};
use tokio::task;

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
    r#type: mod_statuses::RequestType,
}

#[tauri::command]
pub fn connect() {
    /*
    let timeout = Duration::new(5, 0);
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 58430);
    std::thread::sleep(Duration::from_secs(10));
    let mut _stream = TcpStream::connect_timeout(&socket_addr, timeout).expect("foo");
    std::thread::sleep(Duration::from_secs(30));
    println!("connected!");
    let mut i = 212;
    loop {
        let temp = Request {
            id: i,
            code: Some(String::from("kill")),
            message: None,
            parameters: None,
            targets: None,
            viewer: None,
            cost: None,
            r#type: mod_statuses::RequestType::Start,
        };
        let req = serde_json::to_string(&temp).expect("bar") + "\n";
        //let mut buffer: Vec<u8> = Vec::new();
        //let mut reader = std::io::BufReader::new(&stream);

        stream.write(req.as_bytes()).expect("baz");
        stream.flush().expect("bing");

        std::thread::sleep(Duration::from_secs(30));
        i += 1;
        if i > 1000 {
            break;
        }
    }
    */
}

#[tauri::command]
pub async fn listen() {
    println!("testing");
    let listener = TcpListener::bind("127.0.0.1:58430").await.expect("tada");
    let (socket, _) = listener.accept().await.expect("oops");
    let (read_stream, mut write_stream) = socket.into_split();

    task::spawn(listen_and_log(read_stream));

    println!("listening!");
    let mut i = 0;
    loop {
        let temp = Request {
            id: i,
            code: Some(String::from("kill")),
            message: None,
            parameters: Some(Vec::new()),
            targets: None,
            viewer: None,
            cost: None,
            r#type: mod_statuses::RequestType::Start,
        };
        let req = serde_json::to_string(&temp).expect("bar") + "\0";
        write_stream.write_all(req.as_bytes()).await.expect("aaa");
        write_stream.flush().await.expect("flush");
        println!("send!");
        std::thread::sleep(Duration::from_secs(30));
        i += 1;
        if i > 1000 {
            break;
        }
    }
}

async fn listen_and_log(mut shared_socket: OwnedReadHalf) {
    loop {
        let mut buffer = String::new();
        let result = shared_socket
            .read_to_string(&mut buffer)
            .await
            .expect("blargh");
        println!("received: {}", result)
    }
}
