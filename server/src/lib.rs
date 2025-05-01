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

const SERVER: &str = "127.0.0.1:7878";

pub fn run() {
    // TODO should handle errors should this binding have problems
    let listener = TcpListener::bind(SERVER).unwrap();

    let pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("handling connection");
    // let mut buf_reader = BufReader::new(&stream);
    // TODO handle errors gracefully
    // let request: Vec<Result<String, _>> = buf_reader.read_line().lines().collect();
    // println!("{:?}", request);
    let mut request_lines: Vec<String> = Vec::new();
    let mut buf = [0u8; 8192];
    let mut bytes_read = stream.read(&mut buf).unwrap();
    let mut ptr = 0;
    for (i, b) in buf.iter().enumerate() {
        if i >= bytes_read {
            request_lines.push(String::from(
                std::str::from_utf8(&buf[ptr..bytes_read]).unwrap(),
            ));
            break;
        } else if *b == 0xau8 {
            // reached end of the line
            // TODO fix thsi unsafe
            // i-1 to get rid of the carriage return char
            request_lines.push(String::from(std::str::from_utf8(&buf[ptr..i - 1]).unwrap()));
            ptr = i + 1;
        }
    }
    while bytes_read == 8192 {
        bytes_read = stream.read(&mut buf).unwrap();
        println!("read {bytes_read} bytes");
        for (i, b) in buf.iter().enumerate() {
            if i >= bytes_read {
                request_lines.push(String::from(
                    std::str::from_utf8(&buf[ptr..bytes_read]).unwrap(),
                ));
                break;
            } else if *b == 0xau8 {
                // reached end of the line
                // TODO fix thsi unsafe
                // i-1 to get rid of the carriage return char
                request_lines.push(String::from(std::str::from_utf8(&buf[ptr..i - 1]).unwrap()));
                ptr = i + 1;
            }
        }
    }
    println!("{:?}", request_lines);
    let (status_line, filename) = match request_lines[0].as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "./server/index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "./server/index.html")
        }
        "GET /chat HTTP/1.1" => {
            let expected = [
                "GET /chat HTTP/1.1",
                &format!("Host: {SERVER}"),
                "Upgrade: websocket",
                "Connection: Upgrade",
                "Sec-WebSocket-Key: {key}",
                "Sec-WebSocket-Version: 13",
            ];
            if request_lines
                .iter()
                .enumerate()
                .fold(true, |acc, (i, line)| {
                    if i != 4 {
                        acc && &expected[i] == line
                    } else {
                        acc
                    }
                })
            {
                let key = request_lines[4].split(" ").nth(1).unwrap();
                send_handshake(key, &mut stream);
                loop {
                    // TODO make this not suspiciuos
                    let mut read = stream.read(&mut buf).unwrap();
                    // println!("{:?}", buf);
                    let message = websocket::WsMessage::deserialize(buf[..read].to_vec());
                    println!("{:?}", message);
                    //     .get_payload_string();
                    // let mut line = String::from(message);
                    // while read == 8192 {
                    //     read = stream.read(&mut buf).unwrap();
                    //     line.push_str(
                    //         websocket::WsMessage::deserialize(buf[..read].to_vec())
                    //             .get_payload_string()
                    //             .as_str(),
                    //     );
                    // }
                    // println!("{}", line);
                    thread::sleep(std::time::Duration::from_secs(1));
                }
                // ("HTTP/1.1 200 OK", "./server/chat.html")
            } else {
                ("HTTP/1.1 404 NOT FOUND", "./server/404.html")
            }
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "./server/404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}\r\n");
    // this is not good either lol
    stream.write_all(response.as_bytes()).unwrap();
}

fn send_handshake(key: &str, stream: &mut TcpStream) {
    let status_line = "HTTP/1.1 101 Switching Protocols";
    let ws_accept =
        base64::encode(sha1::SHA1Context::new().hash(format!("{key}{}", websocket::GUID)));
    let header_fields = format!(
        "Upgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {ws_accept}\r\nSec-WebSocket-Protocol: chat"
    );
    let handshake = format!("{status_line}\r\n{header_fields}\r\n\r\n");
    stream.write_all(&handshake.as_bytes()).unwrap();
    stream.flush().unwrap();
    println!("handshake sent: {}", handshake);
}
