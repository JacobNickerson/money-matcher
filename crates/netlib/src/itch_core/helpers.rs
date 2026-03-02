pub fn encode_u48(value: u64) -> [u8; 6] {
    let bytes = value.to_be_bytes();
    let mut out = [0u8; 6];
    out.copy_from_slice(&bytes[2..]);
    out
}

pub fn decode_u48(ts: [u8; 6]) -> u64 {
    let mut buf = [0u8; 8];
    buf[2..].copy_from_slice(&ts);
    u64::from_be_bytes(buf)
}

pub fn encode_price(price: f64) -> u32 {
    (price * 10_000.0).round() as u32
}

pub fn decode_price(price: u32) -> f64 {
    price as f64 / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_u48_basic_values() {
        let v1 = 1u64;
        let v12 = 12u64;
        let v123 = 123u64;
        let v1234 = 1234u64;
        let v12345 = 12345u64;

        assert_eq!(encode_u48(v1), [0, 0, 0, 0, 0, 1]);
        assert_eq!(encode_u48(v12), [0, 0, 0, 0, 0, 12]);
        assert_eq!(encode_u48(v123), [0, 0, 0, 0, 0, 123]);
        assert_eq!(encode_u48(v1234), [0, 0, 0, 0, 4, 210]);
        assert_eq!(encode_u48(v12345), [0, 0, 0, 0, 48, 57]);
    }

    #[test]
    fn test_decode_u48_basic_values() {
        let b1 = [0, 0, 0, 0, 0, 1];
        let b12 = [0, 0, 0, 0, 0, 12];
        let b123 = [0, 0, 0, 0, 0, 123];
        let b1234 = [0, 0, 0, 0, 4, 210];
        let b12345 = [0, 0, 0, 0, 48, 57];

        assert_eq!(decode_u48(b1), 1);
        assert_eq!(decode_u48(b12), 12);
        assert_eq!(decode_u48(b123), 123);
        assert_eq!(decode_u48(b1234), 1234);
        assert_eq!(decode_u48(b12345), 12345);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let v1 = 1u64;
        let v12 = 12u64;
        let v123 = 123u64;
        let v1234 = 1234u64;
        let v12345 = 12345u64;

        assert_eq!(decode_u48(encode_u48(v1)), v1);
        assert_eq!(decode_u48(encode_u48(v12)), v12);
        assert_eq!(decode_u48(encode_u48(v123)), v123);
        assert_eq!(decode_u48(encode_u48(v1234)), v1234);
        assert_eq!(decode_u48(encode_u48(v12345)), v12345);
    }
}
