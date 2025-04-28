pub struct SHA1Context {
    hash_vals: [u32; 5],
    curr_chunk_idx: usize, // int_least16_t is surely not necessary for this
    curr_chunk: [u8; 64],
}

impl SHA1Context {
    pub fn initialize_hash() -> SHA1Context {
        SHA1Context {
            hash_vals: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            curr_chunk_idx: 0,
            curr_chunk: [0; 64],
        }
    }

    fn pad_hash(&mut self, mes_len: u64) {
        if self.curr_chunk_idx > 55 {
            self.curr_chunk[self.curr_chunk_idx] = 0x80;
            self.curr_chunk_idx += 1;
            while self.curr_chunk_idx < 64 {
                self.curr_chunk[self.curr_chunk_idx] = 0;
                self.curr_chunk_idx += 1;
            }
            self.hash_chunk();
            self.curr_chunk_idx = 0;
            while self.curr_chunk_idx < 56 {
                self.curr_chunk[self.curr_chunk_idx] = 0;
                self.curr_chunk_idx += 1;
            }
            for i in (0..8).rev() {
                self.curr_chunk[self.curr_chunk_idx] = ((mes_len >> (i * 8)) & 0xff) as u8;
            }
        } else {
            self.curr_chunk[self.curr_chunk_idx] = 0x80;
            self.curr_chunk_idx += 1;
            while self.curr_chunk_idx < 56 {
                self.curr_chunk[self.curr_chunk_idx] = 0;
                self.curr_chunk_idx += 1;
            }
            for i in (0..8).rev() {
                self.curr_chunk[self.curr_chunk_idx] = ((mes_len >> (i * 8)) & 0xff) as u8;
                self.curr_chunk_idx += 1;
            }
        }
        self.hash_chunk();
    }

    pub fn hash(&mut self, message: String) -> String {
        let mes_len = message.len() as u64;
        for b in message.as_bytes() {
            self.curr_chunk[self.curr_chunk_idx] = *b;
            self.curr_chunk_idx += 1;
            if self.curr_chunk_idx == 64 {
                self.hash_chunk();
                self.curr_chunk_idx = 0;
            }
        }
        self.pad_hash(mes_len);
        self.hash_chunks_to_string()
    }

    fn hash_chunk(&mut self) {
        let mut w: [u32; 80] = [0; 80];
        for j in 0..16 {
            w[j] = ((self.curr_chunk[(4 * j) as usize] as u32) << 24) as u32
                | ((self.curr_chunk[4 * j + 1 as usize] as u32) << 16) as u32
                | ((self.curr_chunk[4 * j + 2 as usize] as u32) << 8) as u32
                | (self.curr_chunk[4 * j + 3 as usize]) as u32;
        }
        for j in 16..80 {
            w[j] = Self::leftrotate(w[j - 3] ^ w[j - 8] ^ w[j - 14] ^ w[j - 16], 1);
        }
        let mut a: u32 = self.hash_vals[0];
        let mut b: u32 = self.hash_vals[1];
        let mut c: u32 = self.hash_vals[2];
        let mut d: u32 = self.hash_vals[3];
        let mut e: u32 = self.hash_vals[4];

        for j in 0..80 {
            let (f, k) = match j {
                0..20 => (Self::choose(b, c, d), 0x5A827999),
                20..40 => (Self::parity(b, c, d), 0x6ED9EBA1),
                40..60 => (Self::majority(b, c, d), 0x8F1BBCDC),
                _ => (Self::parity(b, c, d), 0xCA62C1D6),
            };
            let temp: u32 = Self::leftrotate(a, 5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[j]);
            e = d;
            d = c;
            c = Self::leftrotate(b, 30);
            b = a;
            a = temp;
        }
        self.hash_vals[0] = self.hash_vals[0].wrapping_add(a);
        self.hash_vals[1] = self.hash_vals[1].wrapping_add(b);
        self.hash_vals[2] = self.hash_vals[2].wrapping_add(c);
        self.hash_vals[3] = self.hash_vals[3].wrapping_add(d);
        self.hash_vals[4] = self.hash_vals[4].wrapping_add(e);
        self.curr_chunk_idx = 0;
    }
    fn parity(b: u32, c: u32, d: u32) -> u32 {
        b ^ c ^ d
    }
    fn choose(b: u32, c: u32, d: u32) -> u32 {
        // TODO xor vs or
        (b & c) | ((!b) & d)
    }
    fn majority(b: u32, c: u32, d: u32) -> u32 {
        // TODO xor vs or
        (b & c) | (b & d) | (c & d)
    }
    /// Rotate a u32 left by the amount specified.
    pub fn leftrotate(n: u32, amount: u8) -> u32 {
        let msb: u32 = n >> (32 - amount);
        (n << amount) | msb
    }

    fn hash_chunks_to_string(&self) -> String {
        println!("{:?}", self.hash_vals);
        // TODO fix the unwrap
        let mut res_string = String::with_capacity(20);
        for i in 0..5 {
            res_string.push_str(format!("{:x}", self.hash_vals[i]).as_str());
        }
        res_string
        // String::from_utf8(res.to_vec()).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn hash_rfctests() {
        let test1 = String::from("abc");
        let test2a = String::from("abcdbcdecdefdefgefghfghighijhi");
        let test2b = String::from("jkijkljklmklmnlmnomnopnopq");
        let mut test2 = String::new();
        test2.push_str(&test2a);
        test2.push_str(&test2b);
        let test3 = String::from("a");
        let test4a = String::from("01234567012345670123456701234567");
        let mut test4 = String::from("01234567012345670123456701234567");
        test4.push_str(&test4a);
        let testarr: [String; 4] = [test1, test2, test3, test4];
        let res: Vec<String> = testarr
            .iter()
            .map(|test| SHA1Context::initialize_hash().hash(test.to_string()))
            .collect();

        let expected = [
            "A9993E364706816ABA3E25717850C26C9CD0D89D",
            "84983E441C3BD26EBAAE4AA1F95129E5E54670F1",
            "34AA973CD4C4DAA4F61EEB2BDBAD27316534016F",
            "DEA356A2CDDD90C7A7ECEDC5EBB563934F460452",
        ];
        let expected: Vec<String> = expected.iter().map(|s| s.to_lowercase()).collect();

        for i in 0..4 {
            assert_eq!(res[i], expected[i]);
        }
    }
    #[test]
    fn hash_short() {
        let test1 = String::from("hello");
        let test2 = String::from("let's get rusty");
        let test3 = String::from("");
        let test4 = String::from("Kim");
        assert_eq!(
            SHA1Context::initialize_hash().hash(test1),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );
        assert_eq!(
            SHA1Context::initialize_hash().hash(test2),
            "a2590e2ad169b79e91c4c8fcc804f7769d8d7f2c"
        );
        assert_eq!(
            SHA1Context::initialize_hash().hash(test3),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            SHA1Context::initialize_hash().hash(test4),
            "83db02e1cba58c43d01116c50014913b47fa473b"
        );
    }

    #[test]
    fn leftrotate_1() {
        let simple = 0b1;
        assert_eq!(SHA1Context::leftrotate(simple, 1), 0b10);
        let long = 0b10000000000000001;
        assert_eq!(SHA1Context::leftrotate(long, 1), 0b100000000000000010);
        let wrap = 0b10000000000000000000000000000000;
        assert_eq!(SHA1Context::leftrotate(wrap, 1), 0b1);
    }
    #[test]
    fn leftrotate_5() {
        let simple = 0b100000;
        assert_eq!(SHA1Context::leftrotate(simple, 5), 0b10000000000);
        let long = 0b11011;
        assert_eq!(SHA1Context::leftrotate(long, 5), 0b1101100000);
        let wrap = 0b10000000000000000000000000000000;
        assert_eq!(SHA1Context::leftrotate(wrap, 5), 0b10000);
    }
}
