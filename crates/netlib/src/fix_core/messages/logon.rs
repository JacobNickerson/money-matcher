use std::str::from_utf8;

use crate::fix_core::{
    iterator::FixIterator,
    messages::{
        FIX_MESSAGE_TYPE_LOGON, FIXMessage, TAG_ENCRYPT_METHOD, TAG_HEART_BT_INT,
        types::EncryptMethod,
    },
};

/// Logon
/// The logon message authenticates a user establishing a connection to a remote system.
///
/// MsgType = A
#[derive(Debug, Clone)]
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

impl FIXMessage for Logon {
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

    fn from_bytes(msg: &[u8]) -> Result<Self, &'static str> {
        let mut encrypt_method: Option<EncryptMethod> = None;
        let mut heart_bt_int: Option<u16> = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_ENCRYPT_METHOD => {
                    encrypt_method = value
                        .first()
                        .copied()
                        .and_then(|b| EncryptMethod::try_from(b).ok());
                }
                TAG_HEART_BT_INT => {
                    heart_bt_int = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                _ => {}
            }
        }

        Ok(Logon {
            encrypt_method: encrypt_method.unwrap_or_default(),
            heart_bt_int: heart_bt_int.unwrap_or(30),
        })
    }
}
