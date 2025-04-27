pub enum OpCode {
    Cont = 0,
    Text = 1,
    Bin = 2,
    Close = 8,
    Ping = 9,
    Pong = 10,
}

pub const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WsMessage {
    frame_header: u8, // fin, rsv1, rsv2, rsv3, opcode
    p_len: u8,
    ext_p_len: Option<[u8; 6]>, // possible extended p_len + masking
    payload: Box<Vec<u8>>,      // starts with [u8, 2] for masking
}

impl WsMessage {
    fn is_fin(&self) -> bool {
        self.frame_header >> 7 == 1
    }
    fn rsv_bits(&self) -> u8 {
        (self.frame_header >> 4) & 0x7
    }
    fn is_masked(&self) -> bool {
        true
    }
    fn get_frame_type(&self) -> OpCode {
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
    fn get_p_len(&self) -> usize {
        return match self.p_len & 0b0111111 {
            0..126 => (self.p_len & 0b0111111) as usize,
            126 => {
                if let Some(pl) = self.ext_p_len {
                    return (pl[0] as usize) << 8 | (pl[1] as usize);
                } else {
                    return 0;
                }
            }
            127 => {
                if let Some(pl) = self.ext_p_len {
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
    fn get_payload_raw(&self) -> Vec<u8> {
        let p_len: usize = self.get_p_len();
        if p_len > 0 {
            let mut p_buffer = Vec::with_capacity(p_len);
            for i in 0..p_len {
                p_buffer[i] = self.payload[0];
            }
            return p_buffer;
        }
        panic!(); // TODO
    }
    fn get_payload_string(&self) -> String {
        String::from_utf8(self.get_payload_raw()).unwrap()
    }
}
