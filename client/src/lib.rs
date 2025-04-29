use std::{
    io::prelude::*,
    net::{Shutdown, TcpStream},
    str,
};

use ws_utils::{
    base64, sha1, websocket,
    websocket::{FrameHeaderOpt, OpCode},
};

static SERVER: &str = "127.0.0.1:7878";

pub fn run() {
    println!("Started client!");
    let mut client = TcpStream::connect(SERVER).unwrap();
    //TODO do something more safe
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    // let message: String = format!(
    //     "GET /chat HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13"
    // );
    // client.write(message.as_bytes()).unwrap();
    // let mut buf: [u8; 128] = [0; 128];
    // client.read(&mut buf).unwrap();
    send_handshake(key, &mut client);
    let mut recv_response = false;
    while !recv_response {
        //GET response,
        if let Some(response) = get_response(&mut client) {
            if is_valid_response(key, response.as_str()) {
                println!("verified can now start yapping");
                recv_response = true;
            }
        }
    }
    let mut channel_open = true;
    while channel_open {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.as_str() {
            "/stop" => {
                send_close();
                channel_open = false;
            }
            _ => send_text_msg(&input, &mut client),
        }
    }
    client.shutdown(Shutdown::Read).unwrap();
}

fn send_handshake(key: &str, client: &mut TcpStream) {
    let request_line = "GET /chat HTTP/1.1";
    let header_fields = format!(
        "Host: {SERVER}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13"
    );
    let client_handshake = format!("{request_line}\r\n{header_fields}");
    client.write(client_handshake.as_bytes()).unwrap();
}

// TODO make this more robust like the RFC spec
fn is_valid_response(key: &str, response: &str) -> bool {
    let expected_key =
        base64::encode(sha1::SHA1Context::new().hash(format!("{key}{}", websocket::GUID)));
    let expected_lines = [
        "HTTP/1.1 101 Switching Protocols",
        "Upgrade: websocket",
        "Connection: Upgrade",
        &format!("Sec-WebSocket-Accept: {expected_key}"),
        "Sec-WebSocket-Protocol: chat",
    ];
    response
        .split("\r\n")
        .enumerate()
        .fold(true, |acc, (i, res_line)| {
            acc && expected_lines[i] == res_line
        })
}

fn get_response(client: &mut TcpStream) -> Option<String> {
    let mut res = String::new();
    let mut buf: [u8; 4096] = [0; 4096];
    let mut buf_read = client.read(&mut buf).unwrap();
    res.push_str(str::from_utf8(&buf).unwrap());
    while buf_read != 0 {
        buf_read = client.read(&mut buf).unwrap();
        for i in buf_read..4096 {
            buf[i] = 0;
        }
        res.push_str(str::from_utf8(&buf).unwrap());
    }
    if res.len() > 0 { Some(res) } else { None }
}

// TODO fix this devious code
fn send_text_msg(message: &str, client: &mut TcpStream) {
    // SEND MESSAGE
    let mut msg_ptr = 0usize;
    let mut is_first_frame = true;
    let mut ws_message: Option<websocket::WsMessage> = None;
    if message.len() > u16::MAX as usize {
        while message.len() - msg_ptr >= u64::MAX as usize {
            let mut frame_header: u8 = if is_first_frame {
                is_first_frame = false;
                OpCode::Text as u8
            } else {
                OpCode::Cont as u8
            };
            if message.len() - msg_ptr == u64::MAX as usize {
                frame_header |= FrameHeaderOpt::FIN as u8;
            }
            ws_message = Some(websocket::WsMessage::new(
                frame_header,
                &message[msg_ptr..msg_ptr + u64::MAX as usize],
            ));
            msg_ptr += u64::MAX as usize;
        }
        while message.len() - msg_ptr > u16::MAX as usize {
            let mut frame_header: u8 = if is_first_frame {
                is_first_frame = false;
                OpCode::Text as u8
            } else {
                OpCode::Cont as u8
            };
            if message.len() - msg_ptr == u16::MAX as usize {
                frame_header |= FrameHeaderOpt::FIN as u8;
            }
            ws_message = Some(websocket::WsMessage::new(
                frame_header,
                &message[msg_ptr..msg_ptr + u16::MAX as usize],
            ));
            msg_ptr += u16::MAX as usize;
        }
        if message.len() - msg_ptr > 0 {
            let mut frame_header: u8 = if is_first_frame {
                OpCode::Text as u8
            } else {
                OpCode::Cont as u8
            };
            frame_header |= FrameHeaderOpt::FIN as u8;
            ws_message = Some(websocket::WsMessage::new(
                frame_header,
                &message[msg_ptr..message.len()],
            ));
        }
    } else {
        ws_message = Some(websocket::WsMessage::new(
            FrameHeaderOpt::FIN | OpCode::Text,
            message,
        ));
    }
    if let Some(ws_message) = ws_message {
        client.write(&ws_message.serialize()[..]).unwrap();
    } else {
        println!("lol didn't send");
    }
}

fn send_close() {
    println!("closing...");
    // TODO
}
