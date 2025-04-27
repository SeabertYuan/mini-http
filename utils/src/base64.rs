const ALPHABET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '+', '/',
];

pub fn encode(input: String) -> String {
    let in_bytes = input.as_bytes();
    let n_bytes = input.len();
    let n_groups = n_bytes / 3;
    let res_size = (n_bytes + 2) / 3 * 4;
    let mut res = vec![0; res_size];
    println!("{}", input);
    for i in 0..n_groups {
        let mut leftover: u8 = 0;
        for j in 0..3 {
            let idx: usize = i * 4 + j;
            let curr_char = in_bytes[i * 3 + j];
            if j > 1 {
                res[idx] = ALPHABET[(leftover << 2 | (curr_char >> 6)) as usize] as u8;
                res[idx + 1] = ALPHABET[(curr_char & 0b111111) as usize] as u8;
                leftover = 0;
            } else if j > 0 {
                res[idx] = ALPHABET[((leftover << 4) | (curr_char >> 4)) as usize] as u8;
                leftover = curr_char & 0b1111;
            } else {
                res[idx] = ALPHABET[(curr_char >> 2) as usize] as u8;
                leftover = curr_char & 0b11;
            }
        }
    }
    let part_group = 3 - n_bytes % 3;
    match part_group {
        1 => {
            println!("{}", res_size);
            res[res_size - 4] = ALPHABET[(in_bytes[n_bytes - 2] >> 2) as usize] as u8;
            res[res_size - 3] = ALPHABET
                [(((in_bytes[n_bytes - 2] & 0b11) << 4) | (in_bytes[n_bytes - 1] >> 4)) as usize]
                as u8;
            res[res_size - 2] = ALPHABET[((in_bytes[n_bytes - 1] & 0b1111) << 2) as usize] as u8;
            res[res_size - 1] = b'=';
        }
        2 => {
            res[res_size - 4] = ALPHABET[(in_bytes[n_bytes - 1] >> 2) as usize] as u8;
            res[res_size - 3] = ALPHABET[((in_bytes[n_bytes - 1] & 0b11) << 4) as usize] as u8;
            res[res_size - 2] = b'=';
            res[res_size - 1] = b'=';
        }
        _ => (),
    }
    // TODO make htis saafe
    String::from_utf8(res).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_short() {
        let test1 = String::from("Hello World!");
        let test2 = String::from("I love Sohee!");
        let test3 = String::from("Today was an excellent day");
        let test4 = String::from("apple");
        assert_eq!(encode(test1), "SGVsbG8gV29ybGQh");
        assert_eq!(encode(test2), "SSBsb3ZlIFNvaGVlIQ==");
        assert_eq!(encode(test3), "VG9kYXkgd2FzIGFuIGV4Y2VsbGVudCBkYXk=");
        assert_eq!(encode(test4), "YXBwbGU=");
    }

    #[test]
    fn encode_padding() {
        let perfect = String::from("abc");
        let offbyeight = String::from("ab");
        let offbysixteen = String::from("a");
        assert_eq!(encode(perfect), "YWJj");
        assert_eq!(encode(offbyeight), "YWI=");
        assert_eq!(encode(offbysixteen), "YQ==");
    }

    #[test]
    fn encode_complex() {
        let special = String::from("\\$*!)(@#*$&%^&");
        let special2 = String::from("\\r\\n\\tsl~`1234567890{}|/.,<>?");
        assert_eq!(encode(special), "XCQqISkoQCMqJCYlXiY=");
        assert_eq!(encode(special2), "XHJcblx0c2x+YDEyMzQ1Njc4OTB7fXwvLiw8Pj8=");
    }

    #[test]
    fn encode_long() {
        let the_raven = std::fs::read_to_string("test_resources/the_raven.txt").unwrap();
        let expected = std::fs::read_to_string("test_resources/b64_raven.txt").unwrap();
        assert_eq!(
            encode(String::from(the_raven.trim_end())),
            expected.trim_end()
        );
    }
}
