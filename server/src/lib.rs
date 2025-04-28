pub mod threadpool;

use ws_utils::{base64, sha1, websocket};

use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use crate::threadpool::ThreadPool;

pub fn run() {
    // TODO should handle errors should this binding have problems
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    // TODO handle erros gracefully
    let request_line: String = buf_reader.lines().next().unwrap().unwrap();
    println!("{}", request_line);
    let (status_line, filename) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "./server/index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "./server/index.html")
        }
        "GET /chat HTTP/1.1" => ("HTTP/1.1 200 OK", "./server/chat.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "./server/404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}\r\n");
    // this is not good either lol
    stream.write_all(response.as_bytes()).unwrap();
}

fn send_handshake(key: &str) {
    let status_line = "HTTP/1.1 101 Switching Protocols";
    let ws_accept = base64::encode(
        sha1::SHA1Context::initialize_hash().hash(format!("{key}{}", websocket::GUID)),
    );
    let header_fields = format!(
        "Upgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {ws_accept}\r\nSec-WebSocket-Protocol: chat"
    );
    let handshake = format!("{status_line}\r\n{header_fields}\r\n");
    // TODO send handshake
}
