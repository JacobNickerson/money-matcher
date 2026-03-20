use crate::itch_core::helpers::{decode_u48, encode_u48};
use crate::itch_core::messages::ItchMessage;
use std::str::from_utf8;
use zerocopy::byteorder::{BigEndian, U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Base {
    pub(crate) message_type: u8,
    pub stock_locate: U16<BigEndian>,
    pub tracking_number: U16<BigEndian>,
    pub timestamp: [u8; 6],
}

impl Base {
    pub fn new() -> Self {
        Self {
            message_type: ITCH_MESSAGE_TYPE_,
        }
    }

    pub fn print(&self) {
        println!(
            "ITCH Message: Base | stock_locate={} | tracking_number={} | timestamp={:?} ",
            self.stock_locate.get(),
            self.tracking_number.get(),
            decode_u48(self.timestamp),
        );
    }
}

impl ItchMessage for Base {
    fn set_tracking_number(&mut self, n: u16) {
        self.tracking_number = U16::new(n);
    }

    fn set_stock_locate(&mut self, n: u16) {
        self.stock_locate = U16::new(n);
    }
}
