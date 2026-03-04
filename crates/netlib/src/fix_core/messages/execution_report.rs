#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionReport {}

impl ExecutionReport {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::new();

        // 11 - ClOrdID
        buf.extend_from_slice(b"11=");
        buf.extend_from_slice(itoa_buf.format(123123123).as_bytes());
        buf.push(0x01);

        buf
    }
}
