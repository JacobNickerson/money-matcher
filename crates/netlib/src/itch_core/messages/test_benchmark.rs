use crate::itch_core::helpers::encode_u48;
use crate::itch_core::messages::{ItchMessage, MESSAGE_TYPE_TEST_BENCHMARK};
use zerocopy::byteorder::{BigEndian, U16};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct TestBenchmark {
    pub(crate) message_type: u8,
    pub timestamp: [u8; 6],
    pub tracking_number: U16<BigEndian>,
    pub stock_locate: U16<BigEndian>,
}

impl TestBenchmark {
    pub fn new(timestamp: u64) -> Self {
        Self {
            message_type: MESSAGE_TYPE_TEST_BENCHMARK,
            timestamp: encode_u48(timestamp),
            tracking_number: U16::new(0),
            stock_locate: U16::new(0),
        }
    }
}

impl ItchMessage for TestBenchmark {
    fn set_tracking_number(&mut self, n: u16) {
        self.tracking_number = U16::new(n);
    }

    fn set_stock_locate(&mut self, n: u16) {
        self.stock_locate = U16::new(n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_benchmark_initial_state() {
        let msg = TestBenchmark::new(1000);

        assert_eq!(msg.message_type, MESSAGE_TYPE_TEST_BENCHMARK);
        assert_eq!(msg.tracking_number.get(), 0);
        assert_eq!(msg.stock_locate.get(), 0);
    }

    #[test]
    fn test_test_benchmark_trait_updates() {
        let mut msg = TestBenchmark::new(0);

        msg.set_tracking_number(5);
        msg.set_stock_locate(10);

        assert_eq!(msg.tracking_number.get(), 5);
        assert_eq!(msg.stock_locate.get(), 10);
    }
}
