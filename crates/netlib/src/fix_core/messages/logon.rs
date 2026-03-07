use crate::fix_core::messages::{
    FIX_MESSAGE_TYPE_LOGON, FixMessage, TAG_ENCRYPT_METHOD, TAG_HEART_BT_INT, types::EncryptMethod,
};

/// Logon
/// The logon message authenticates a user establishing a connection to a remote system.
///
/// MsgType = A
pub struct Logon {
    pub encrypt_method: EncryptMethod,
    /// Same value used by both sides
    pub heart_bt_int: u16,
}

impl Logon {
    pub fn new(encrypt_method: EncryptMethod, heart_bt_int: u16) -> Self {
        Self {
            encrypt_method,
            heart_bt_int,
        }
    }
}

impl FixMessage for Logon {
    const MESSAGE_TYPE: &'static [u8] = FIX_MESSAGE_TYPE_LOGON;

    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::new();

        // 98 - EncryptMethod
        buf.extend_from_slice(TAG_ENCRYPT_METHOD);
        buf.push(b'=');
        buf.push(self.encrypt_method as u8);
        buf.push(0x01);

        // 108 - HeartBtInt
        buf.extend_from_slice(TAG_HEART_BT_INT);
        buf.push(b'=');
        buf.extend_from_slice(itoa_buf.format(self.heart_bt_int).as_bytes());
        buf.push(0x01);

        buf
    }
}
