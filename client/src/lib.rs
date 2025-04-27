use std::{
    io::prelude::*,
    net::{Shutdown, TcpStream},
    str,
};

use ws_utils::{base64, sha1, websocket};

static SERVER: &str = "127.0.0.1:7878";

pub fn run() {
    println!("Started client!");
    let mut client = TcpStream::connect(SERVER).unwrap();
    //TODO do something more safe
    let message: &str = "GET /chat HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13";
    client.write(message.as_bytes()).unwrap();
    let mut buf: [u8; 128] = [0; 128];
    client.read(&mut buf).unwrap();
    send_handshake("dGhlIHNhbXBsZSBub25jZQ==");
    client.shutdown(Shutdown::Read).unwrap();
    println!("{}", str::from_utf8(&buf).unwrap());
}

fn send_handshake(key: &str) -> String {
    let request_line = "GET /chat HTTP/1.1";
    let header_fields = format!(
        "Host: {SERVER}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13"
    );
    let client_handshake = format!("{request_line}\r\n{header_fields}");
    //TODO send the handshake
    base64::encode(sha1::hash(format!("{key}{}", websocket::GUID)))
}
