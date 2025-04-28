pub fn hash(input: String) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;
    let in_len: u64 = input.len() as u64;
    // padding (input length + 8 bytes from length + 1 byte for 1 bit) to align to 512 bits
    let pad_len: u64 = 64 - (in_len + 9) % 64;
    let total_byte_len = pad_len + in_len + 9;
    let mut m_bytes: Vec<u8> = Vec::with_capacity(total_byte_len as usize);
    for c in input.as_bytes() {
        m_bytes.push(*c);
    }
    m_bytes.push(0x80);
    while m_bytes.len() < (total_byte_len - 8) as usize {
        m_bytes.push(0);
    }
    for i in (0..8).rev() {
        m_bytes.push(((in_len >> i * 8) & 0xff) as u8);
    }
    // 512 bit chunks
    let n_chunks = total_byte_len / 64;
    println!("{:?}", m_bytes);
    println!("{:?}", n_chunks);
    for i in 0..n_chunks {
        let mut w: [u32; 80] = [0; 80];
        for j in 0..16 {
            w[j] = ((m_bytes[(4 * j) + 64 * i as usize] as u32) << 24) as u32
                | ((m_bytes[4 * j + 1 + 64 * i as usize] as u32) << 16) as u32
                | ((m_bytes[4 * j + 2 + 64 * i as usize] as u32) << 8) as u32
                | (m_bytes[4 * j + 3 + 64 * i as usize]) as u32;
        }
        for j in 16..80 {
            w[j] = leftrotate(w[j - 3] ^ w[j - 8] ^ w[j - 14] ^ w[j - 16], 1);
        }
        let mut a: u32 = h0;
        let mut b: u32 = h1;
        let mut c: u32 = h2;
        let mut d: u32 = h3;
        let mut e: u32 = h4;
        let mut f: u32;
        let mut k: u32;

        for j in 0..80 {
            if j < 20 {
                // TODO xor vs or
                f = (b & c) ^ ((!b) & d);
                k = 0x5A827999;
            } else if j < 40 {
                f = b ^ c ^ d;
                k = 0x6ED9EBA1;
            } else if j < 60 {
                f = (b & c) ^ (b & d) ^ (c & d);
                k = 0x8F1BBCDC;
            } else {
                f = b ^ c ^ d;
                k = 0xCA62C1D6;
            }
            let temp: u32 = leftrotate(a, 5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[j]);
            e = d;
            d = c;
            c = leftrotate(b, 30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }
    hash_chunks_to_string([h0, h1, h2, h3, h4])
}

/// Rotate a u32 left by the amount specified.
fn leftrotate(n: u32, amount: u8) -> u32 {
    let msb: u32 = n >> (32 - amount);
    (n << amount) | msb
}

fn hash_chunks_to_string(chunks: [u32; 5]) -> String {
    println!("{:?}", chunks);
    // TODO fix the unwrap
    let mut res_string = String::with_capacity(20);
    for i in 0..5 {
        res_string.push_str(format!("{:x}", chunks[i]).as_str());
    }
    res_string
    // String::from_utf8(res.to_vec()).unwrap()
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
        let res: Vec<String> = testarr.iter().map(|test| hash(test.to_string())).collect();

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
        assert_eq!(hash(test1), "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
        assert_eq!(hash(test2), "a2590e2ad169b79e91c4c8fcc804f7769d8d7f2c");
        assert_eq!(hash(test3), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(hash(test4), "83db02e1cba58c43d01116c50014913b47fa473b");
    }

    #[test]
    fn leftrotate_1() {
        let simple = 0b1;
        assert_eq!(leftrotate(simple, 1), 0b10);
        let long = 0b10000000000000001;
        assert_eq!(leftrotate(long, 1), 0b100000000000000010);
        let wrap = 0b10000000000000000000000000000000;
        assert_eq!(leftrotate(wrap, 1), 0b1);
    }
    #[test]
    fn leftrotate_5() {
        let simple = 0b100000;
        assert_eq!(leftrotate(simple, 5), 0b10000000000);
        let long = 0b11011;
        assert_eq!(leftrotate(long, 5), 0b1101100000);
        let wrap = 0b10000000000000000000000000000000;
        assert_eq!(leftrotate(wrap, 5), 0b10000);
    }
}
