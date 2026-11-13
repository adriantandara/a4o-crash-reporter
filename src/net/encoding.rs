const BASE64_URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn base64_url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n =
            (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        let symbols = chunk.len() + 1;
        for i in 0..symbols {
            out.push(BASE64_URL[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

pub fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    for c in text.trim().bytes().filter(|&c| c != b'=') {
        let value = match c {
            b'+' => 62,
            b'/' => 63,
            _ => BASE64_URL.iter().position(|&a| a == c)? as u32,
        };
        bits = (bits << 6) | value;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_base64_url_without_padding() {
        assert_eq!(base64_url(b""), "");
        assert_eq!(base64_url(b"f"), "Zg");
        assert_eq!(base64_url(b"fo"), "Zm8");
        assert_eq!(base64_url(b"foo"), "Zm9v");
        assert_eq!(base64_url(&[0xfb, 0xff]), "-_8");
    }

    #[test]
    fn decodes_both_alphabets() {
        assert_eq!(decode_base64("Zm9v").unwrap(), b"foo");
        assert_eq!(decode_base64("Zm8=").unwrap(), b"fo");
        assert_eq!(decode_base64("-_8").unwrap(), [0xfb, 0xff]);
        assert_eq!(decode_base64("+/8=").unwrap(), [0xfb, 0xff]);
        assert!(decode_base64("Zm!v").is_none());
    }

    #[test]
    fn encodes_hex() {
        assert_eq!(hex(&[0x00, 0xab, 0x10]), "00ab10");
    }
}
