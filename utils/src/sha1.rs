pub fn hash(input: String) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;
    let in_len: u64 = input.len() as u64;
    // padding (input length + 8 bytes from length + 1 byte for 1 bit) to align to 512 bits
    let pad_len: u64 = 64 - (in_len + 9) % 64;
    let total_len = pad_len + in_len + 1;
    let mut m_bytes: Vec<u8> = Vec::with_capacity((total_len + 8) as usize);
    for c in input.as_bytes() {
        m_bytes.push(*c);
    }
    m_bytes.push(0x80);
    while m_bytes.len() < total_len as usize {
        m_bytes.push(0);
    }
    for i in (0..8).rev() {
        m_bytes.push(((in_len >> i * 8) & 0xff) as u8);
    }
    // 512 bit chunks
    let n_chunks = (total_len + 8) / 8;
    for i in 0..n_chunks {
        let mut hash_word: [u32; 80] = [0; 80];
        for j in 0..15 {
            hash_word[j] = ((m_bytes[4 * j] as u32) << 24) as u32
                | ((m_bytes[4 * j + 1] as u32) << 16) as u32
                | ((m_bytes[4 * j + 2] as u32) << 8) as u32
                | (m_bytes[4 * j + 3]) as u32;
        }
        for j in 16..80 {
            hash_word[j] = leftrotate(
                hash_word[j - 3] ^ hash_word[j - 8] ^ hash_word[j - 14] ^ hash_word[j - 16],
                1,
            );
        }
        let mut hv0: u32 = h0;
        let mut hv1: u32 = h1;
        let mut hv2: u32 = h2;
        let mut hv3: u32 = h3;
        let mut hv4: u32 = h4;
        let mut hv5: u32 = 0;
        let mut hv6: u32 = 0;

        for j in 0..80 {
            if j < 20 {
                hv5 = (hv1 & hv2) | (!hv1 & hv3);
                hv6 = 0x5A827999;
            } else if j < 40 {
                hv5 = hv1 ^ hv2 ^ hv3;
                hv6 = 0x6ED9EBA1;
            } else if j < 60 {
                hv5 = (hv1 & hv2) ^ (hv1 & hv3) ^ (hv2 & hv3);
                hv6 = 0x8F1BBCDC;
            } else {
                hv5 = hv1 ^ hv2 ^ hv3;
                hv6 = 0xCA62C1D6;
            }
            let temp: u32 = leftrotate(hv0, 5)
                .wrapping_add(hv5)
                .wrapping_add(hv4)
                .wrapping_add(hv6)
                .wrapping_add(hash_word[j]);
            hv4 = hv3;
            hv3 = hv2;
            hv2 = leftrotate(hv1, 30);
            hv1 = hv0;
            hv0 = temp;
        }
        h0 = h0.wrapping_add(hv0);
        h1 = h1.wrapping_add(hv1);
        h2 = h2.wrapping_add(hv2);
        h3 = h3.wrapping_add(hv3);
        h4 = h4.wrapping_add(hv4);
    }
    hash_chunks_to_string([h0, h1, h2, h3, h4])
}

/// Rotate a u32 left by the amount specified.
fn leftrotate(n: u32, amount: u8) -> u32 {
    let msb: u32 = n >> (32 - amount);
    (n << amount) | msb
}

fn hash_chunks_to_string(chunks: [u32; 5]) -> String {
    let mut res: [u8; 20] = [0; 20];
    for i in 0..5 {
        for j in (0..4).rev() {
            res[i * 4 + (3 - j)] = ((chunks[i] >> 8 * j) & 0xff) as u8;
        }
    }
    // TODO fix the unwrap
    let mut res_string = String::with_capacity(20);
    for i in 0..20 {
        res_string.push_str(format!("{:x}", res[i]).as_str());
    }
    res_string
    // String::from_utf8(res.to_vec()).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
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
