use mm_core::{
    itch_core::{
        helpers::decode_u48,
        messages::{
            ITCH_MESSAGE_TYPE_ADD_ORDER, ITCH_MESSAGE_TYPE_ORDER_CANCEL,
            ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE, ITCH_MESSAGE_TYPE_ORDER_REPLACE,
        },
    },
    lob_core::{
        market_events::{L3Event, MarketEvent, MarketEventType, TradeEvent},
        market_orders::{OrderSide, OrderStatus, OrderType},
    },
    moldudp64_core::types::Header,
};
use ringbuf::{HeapProd, traits::Producer};
use std::net::UdpSocket;
use zerocopy::FromBytes;

pub struct ReceiverHandler {
    socket: UdpSocket,
    output: HeapProd<MarketEvent>,
}

impl ReceiverHandler {
    pub fn new(output: HeapProd<MarketEvent>, socket: UdpSocket) -> Self {
        Self { socket, output }
    }

    pub fn run(mut self) {
        let mut buf = [0u8; 2048];

        loop {
            let (len, _) = match self.socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(_) => continue,
            };
            self.handle_packet(&buf[..len]);
        }
    }

    fn handle_packet(&mut self, bytes: &[u8]) {
        let len: usize = bytes.len();
        if len < 20 {
            return;
        }

        let header = match Header::read_from_prefix(bytes) {
            Ok(v) => v.0,
            Err(_) => return,
        };

        let mc = u16::from_be_bytes(header.message_count) as usize;
        let mut offset = 20;

        for _ in 0..mc {
            if !self.handle_message(bytes, len, &mut offset) {
                break;
            }
        }
    }

    fn handle_message(&mut self, bytes: &[u8], len: usize, offset: &mut usize) -> bool {
        if *offset + 2 > len {
            return false;
        }

        let ml = u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]) as usize;
        *offset += 2;

        if *offset + ml > len {
            return false;
        }

        let message_data = &bytes[*offset..*offset + ml];
        *offset += ml;

        if message_data.is_empty() {
            return true;
        }

        let event = match Self::parse_event(message_data) {
            Some(v) => v,
            None => return true,
        };

        self.push_event(event);
        true
    }

    fn parse_event(message_data: &[u8]) -> Option<MarketEvent> {
        if message_data.is_empty() {
            return None;
        }

        match message_data[0] {
            ITCH_MESSAGE_TYPE_ADD_ORDER => {
                // let stock_locate = u16::from_be_bytes(message_data[1..3].try_into().unwrap());
                // let tracking_number = u16::from_be_bytes(message_data[3..5].try_into().unwrap());

                let timestamp = decode_u48(message_data[5..11].try_into().unwrap());

                let order_reference_number =
                    u64::from_be_bytes(message_data[11..19].try_into().unwrap());

                let side = message_data[19];
                let shares = u32::from_be_bytes(message_data[20..24].try_into().unwrap());

                // let stock = &message_data[24..32];

                let price = u32::from_be_bytes(message_data[32..36].try_into().unwrap());

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::L3(L3Event {
                        order_id: order_reference_number,
                        side: side.try_into().unwrap(),
                        timestamp,
                        kind: OrderType::Limit {
                            qty: shares.into(),
                            price: price.into(),
                        },
                    }),
                })
            }
            ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE => {
                // let stock_locate = u16::from_be_bytes(message_data[1..3].try_into().unwrap());
                // let tracking_number = u16::from_be_bytes(message_data[3..5].try_into().unwrap());

                let timestamp = decode_u48(message_data[5..11].try_into().unwrap());

                // let order_reference_number = u64::from_be_bytes(message_data[11..19].try_into().unwrap());

                let executed_shares = u32::from_be_bytes(message_data[19..23].try_into().unwrap());

                // let match_number = u64::from_be_bytes(message_data[23..31].try_into().unwrap());
                // let printable = message_data[31];

                let execution_price = u32::from_be_bytes(message_data[32..36].try_into().unwrap());

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::Trade(TradeEvent {
                        quantity: executed_shares.into(),
                        price: execution_price.into(),
                        aggressor_side: OrderSide::Ask, // PLACEHOLDER
                    }),
                })
            }
            ITCH_MESSAGE_TYPE_ORDER_CANCEL => {
                // let stock_locate = u16::from_be_bytes(message_data[1..3].try_into().unwrap());
                // let tracking_number = u16::from_be_bytes(message_data[3..5].try_into().unwrap());

                let timestamp = decode_u48(message_data[5..11].try_into().unwrap());

                let order_reference_number =
                    u64::from_be_bytes(message_data[11..19].try_into().unwrap());

                // let canceled_shares = u32::from_be_bytes(message_data[19..23].try_into().unwrap());

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::L3(L3Event {
                        order_id: order_reference_number,
                        side: OrderSide::Ask, // PLACEHOLDER
                        timestamp,
                        kind: OrderType::Cancel, // NEEDS QUANTITY
                    }),
                })
            }
            ITCH_MESSAGE_TYPE_ORDER_REPLACE => {
                // let stock_locate = u16::from_be_bytes(message_data[1..3].try_into().unwrap());
                // let tracking_number = u16::from_be_bytes(message_data[3..5].try_into().unwrap());

                let timestamp = decode_u48(message_data[5..11].try_into().unwrap());

                let original_order_reference_number =
                    u64::from_be_bytes(message_data[11..19].try_into().unwrap());
                let new_order_reference_number =
                    u64::from_be_bytes(message_data[19..27].try_into().unwrap());
                let shares = u32::from_be_bytes(message_data[27..31].try_into().unwrap());
                let price = u32::from_be_bytes(message_data[31..35].try_into().unwrap());

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::L3(L3Event {
                        order_id: new_order_reference_number,
                        side: OrderSide::Ask, // PLACEHOLDER
                        timestamp,
                        kind: OrderType::Update {
                            old_id: original_order_reference_number,
                            qty: shares.into(),
                            price: price.into(),
                        },
                    }),
                })
            }
            _ => None,
        }
    }

    fn push_event(&mut self, event: MarketEvent) {
        self.output.try_push(event).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::{
        itch_core::messages::{
            add_order::AddOrder, order_executed_with_price::OrderExecutedWithPrice,
        },
        lob_core::market_orders::OrderType,
    };
    use ringbuf::{
        HeapCons,
        traits::{Consumer, Split},
    };

    fn make_handler() -> (ReceiverHandler, HeapCons<MarketEvent>) {
        let (tx, rx) = ringbuf::HeapRb::<MarketEvent>::new(8).split();
        let socket = UdpSocket::bind("127.0.0.1:0").expect("err");
        (ReceiverHandler::new(tx, socket), rx)
    }

    #[test]
    fn test_parse_event_add_order() {
        let mut stock = [b' '; 8];
        stock[..4].copy_from_slice(b"TEST");

        let mut buf = [0u8; 36];
        AddOrder::encode_into(&mut buf, 1, 12, 123, 5000, b'B', 10, stock, 99);

        let event = ReceiverHandler::parse_event(&buf).expect("err");

        assert_eq!(event.timestamp, 123);
        match event.kind {
            MarketEventType::L3(v) => {
                assert_eq!(v.order_id, 5000);
                match v.kind {
                    OrderType::Limit { qty, price } => {
                        assert_eq!(qty, 10);
                        assert_eq!(price, 99);
                    }
                    _ => panic!("wrong order type"),
                }
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_parse_event_order_executed_with_price() {
        let mut buf = [0u8; 36];
        OrderExecutedWithPrice::encode_into(&mut buf, 1, 1000, 123, 5000, 10, 9999, b'Y', 100);
        let event = ReceiverHandler::parse_event(&buf).expect("err");

        assert_eq!(event.timestamp, 123);
        match event.kind {
            MarketEventType::Trade(v) => {
                assert_eq!(v.quantity, 10);
                assert_eq!(v.price, 100);
            }
            _ => panic!("wrong event"),
        }
    }

    #[test]
    fn test_handle_packet_ignores_short_packet() {
        let (mut h, mut rx) = make_handler();

        let buf = [0u8; 10];
        h.handle_packet(&buf);

        assert!(rx.try_pop().is_none());
    }
}
