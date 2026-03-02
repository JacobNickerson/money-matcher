pub trait FixMessage {
    const MESSAGE_TYPE: &'static [u8];
    fn as_bytes(&self) -> Vec<u8>;
}

pub struct FixFrame {
    pub msg_type: &'static [u8],
    pub body: Vec<u8>,
}

pub mod new_order;
