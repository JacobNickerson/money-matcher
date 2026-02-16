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
