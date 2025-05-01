use std::ops::BitOr;

pub enum OpCode {
    Cont = 0,
    Text = 1,
    Bin = 2,
    Close = 8,
    Ping = 9,
    Pong = 10,
}
impl BitOr for OpCode {
    type Output = u8;
    fn bitor(self, other: OpCode) -> u8 {
        self as u8 | other as u8
    }
}

pub enum FrameHeaderOpt {
    FIN = 0x80,
    RSV1 = 0x40,
    RSV2 = 0x20,
    RSV3 = 0x10,
}
impl BitOr<OpCode> for FrameHeaderOpt {
    type Output = u8;
    fn bitor(self, other: OpCode) -> u8 {
        self as u8 | other as u8
    }
}

pub const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WsMessage {
    frame_header: u8, // fin, rsv1, rsv2, rsv3, opcode
    p_len: u8,
    ext_p_len: Option<Vec<u8>>, // possible extended p_len + masking
    payload: Box<Vec<u8>>,      // starts with [u8, 2] for masking
}

impl std::fmt::Debug for WsMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsMessage")
            .field("frame_header", &self.frame_header)
            .field("p_len", &self.p_len)
            .field("ext_p_len", &self.ext_p_len)
            .field("payload", &std::str::from_utf8(&self.payload))
            .finish()
    }
}

impl WsMessage {
    pub fn new(header: u8, payload: &str) -> WsMessage {
        let mut res = WsMessage {
            frame_header: header,
            p_len: 0,
            ext_p_len: None,
            payload: Box::new(payload.as_bytes().to_vec()),
        };
        let mut ext_bytes = 0;
        if payload.len() > 0xffff {
            res.p_len = 127;
            ext_bytes = 8;
        } else if payload.len() > 125 {
            res.p_len = 126;
            ext_bytes = 2;
        } else {
            res.p_len = payload.len() as u8
        };
        res.ext_p_len = Some(
            (0..ext_bytes)
                .map(|i| ((payload.len() >> ((ext_bytes - i - 1) * 8)) & 0xff) as u8)
                .collect(),
        );
        res
    }
    pub fn deserialize(b_stream: Vec<u8>) -> WsMessage {
        let (ext_p_len, p_start_idx) = if b_stream[1] < 126 {
            (None, 2)
        } else if b_stream[1] == 126 {
            (Some(vec![b_stream[2], b_stream[3]]), 4)
        } else {
            (
                Some(vec![
                    b_stream[2],
                    b_stream[3],
                    b_stream[4],
                    b_stream[5],
                    b_stream[6],
                    b_stream[7],
                    b_stream[8],
                    b_stream[9],
                ]),
                10,
            )
        };
        WsMessage {
            frame_header: b_stream[0],
            p_len: b_stream[1],
            ext_p_len,
            payload: Box::new(b_stream[p_start_idx..].to_vec()),
        }
    }
    pub fn is_fin(&self) -> bool {
        self.frame_header >> 7 == 1
    }
    pub fn rsv_bits(&self) -> u8 {
        (self.frame_header >> 4) & 0x7
    }
    pub fn is_masked(&self) -> bool {
        true
    }
    pub fn get_frame_type(&self) -> OpCode {
        return match self.frame_header & 0xf {
            0 => OpCode::Cont,
            1 => OpCode::Text,
            2 => OpCode::Bin,
            8 => OpCode::Close,
            9 => OpCode::Ping,
            10 => OpCode::Pong,
            _ => unreachable!(),
        };
    }
    // 0 if invalid, otherwise the size of the payload
    pub fn get_p_len(&self) -> usize {
        return match self.p_len & 0b0111111 {
            0..126 => (self.p_len & 0b0111111) as usize,
            126 => {
                if let Some(pl) = &self.ext_p_len {
                    (pl[0] as usize) << 8 | (pl[1] as usize)
                } else {
                    0
                }
            }
            127 => {
                if let Some(pl) = &self.ext_p_len {
                    let mut ans = 0;
                    for i in 0..8 {
                        ans = ans | (pl[i] as usize) << 8 * (7 - i);
                    }
                    return ans;
                } else {
                    return 0;
                }
            }
            _ => 0,
        };
    }
    pub fn get_payload_raw(&self) -> Vec<u8> {
        let p_len: usize = self.get_p_len();
        if p_len > 0 {
            let mut p_buffer = Vec::with_capacity(p_len);
            for i in 0..p_len {
                p_buffer[i] = self.payload[0];
            }
            return p_buffer;
        }
        vec![]
    }
    pub fn get_payload_string(&self) -> String {
        String::from_utf8(self.get_payload_raw()).unwrap()
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.push(self.frame_header);
        res.push(self.p_len);
        if let Some(ext_len) = &self.ext_p_len {
            res.append(&mut ext_len.clone());
        }
        res.append(&mut self.payload.clone());
        res
    }
}
