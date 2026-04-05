use mm_core::{
    itch_core::{
        helpers::decode_u48,
        messages::{
            ITCH_MESSAGE_TYPE_ADD_ORDER, ITCH_MESSAGE_TYPE_ORDER_CANCEL,
            ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE, ITCH_MESSAGE_TYPE_ORDER_REPLACE,
        },
    },
    lob_core::{
        market_events::{L3Event, L3EventExtra, MarketEvent, MarketEventType, TradeEvent},
        market_orders::{OrderSide, OrderType},
    },
};
use ringbuf::{HeapProd, traits::Producer};
use std::{collections::BTreeMap, net::UdpSocket};

/// A UDP receiver that parses MoldUDP64 packets into internal market events.
pub struct ReceiverHandler {
    socket: UdpSocket,
    output: HeapProd<MarketEvent>,
    gap_buffer: BTreeMap<u64, MarketEvent>,
    sequence_number: u64,
}

impl ReceiverHandler {
    /// Initializes a new receiver handler with a designated output queue and UDP socket.
    pub fn new(output: HeapProd<MarketEvent>, socket: UdpSocket) -> Self {
        Self {
            socket,
            output,
            gap_buffer: BTreeMap::new(),
            sequence_number: 0,
        }
    }

    /// Runs the main event loop, polling the socket and spinning on `WouldBlock`.
    pub fn run(mut self) {
        self.socket.set_nonblocking(true).expect("err");
        let mut buf = [0u8; 2048];

        loop {
            let (len, _) = match self.socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::hint::spin_loop();
                    continue;
                }
                Err(_) => continue,
            };
            self.handle_packet(&buf[..len]);
        }
    }

    /// Validates the MoldUDP64 packet header and iterates through the message block.
    #[inline(always)]
    fn handle_packet(&mut self, bytes: &[u8]) {
        let len = bytes.len();

        if len < 20 {
            return;
        }

        let _session_id: &[u8; 10] = match bytes[0..10].try_into() {
            Ok(x) => x,
            Err(_) => return,
        }; // @todo Validate session id

        let seq_num_bytes: &[u8; 8] = match bytes[10..18].try_into() {
            Ok(x) => x,
            Err(_) => return,
        };
        let seq_num = u64::from_be_bytes(*seq_num_bytes);

        if self.sequence_number == 0 {
            self.sequence_number = seq_num;
        }

        let mc_bytes: &[u8; 2] = match bytes[18..20].try_into() {
            Ok(x) => x,
            Err(_) => return,
        };

        let mc = u16::from_be_bytes(*mc_bytes) as usize;
        let mut offset = 20;

        for i in 0..mc {
            let current_msg_seq_num = seq_num + i as u64;

            if let Some(event) = self.handle_message(bytes, len, &mut offset) {
                if current_msg_seq_num == self.sequence_number {
                    self.sequence_number += 1;
                    self.push_event(event);
                } else if current_msg_seq_num > self.sequence_number {
                    self.gap_buffer.insert(current_msg_seq_num, event);
                    // self.resend_request(self.sequence_number)
                }
            } else {
                break;
            }
        }

        while let Some(event) = self.gap_buffer.remove(&self.sequence_number) {
            self.push_event(event);
            self.sequence_number += 1;
        }
    }

    /// Extracts an individual message from the packet and routes it for event parsing.
    #[inline(always)]
    fn handle_message(
        &mut self,
        bytes: &[u8],
        len: usize,
        offset: &mut usize,
    ) -> Option<MarketEvent> {
        if *offset + 2 > len {
            return None;
        }

        let ml = u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]) as usize;
        *offset += 2;

        if *offset + ml > len {
            return None;
        }

        let message_data = &bytes[*offset..*offset + ml];
        *offset += ml;

        if message_data.is_empty() {
            return None;
        }

        let event = match Self::parse_event(message_data) {
            Some(v) => v,
            None => return None,
        };

        Some(event)
    }

    /// Decodes raw ITCH message bytes into the LOB `MarketEvent` format.
    #[inline(always)]
    fn parse_event(message_data: &[u8]) -> Option<MarketEvent> {
        if message_data.is_empty() {
            return None;
        }

        match message_data[0] {
            ITCH_MESSAGE_TYPE_ADD_ORDER => {
                if message_data.len() < 36 {
                    return None;
                }

                let timestamp = decode_u48(message_data[5..11].try_into().ok()?);
                let order_reference_number =
                    u64::from_be_bytes(message_data[11..19].try_into().ok()?);
                let side = message_data[19];
                let shares = u32::from_be_bytes(message_data[20..24].try_into().ok()?);
                let price = u32::from_be_bytes(message_data[32..36].try_into().ok()?);

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::L3(L3Event {
                        order_id: order_reference_number,
                        side: side.try_into().unwrap(),
                        timestamp,
                        kind: if price > 0 {
                            OrderType::Limit {
                                qty: shares.into(),
                                price: price.into(),
                            }
                        } else {
                            OrderType::Market { qty: shares.into() }
                        },
                        extra: L3EventExtra::None,
                    }),
                })
            }
            ITCH_MESSAGE_TYPE_ORDER_EXECUTED_WITH_PRICE => {
                if message_data.len() < 36 {
                    return None;
                }

                let timestamp = decode_u48(message_data[5..11].try_into().ok()?);
                let maker_id = u64::from_be_bytes(message_data[11..19].try_into().ok()?);
                let executed_shares = u32::from_be_bytes(message_data[19..23].try_into().ok()?);
                let execution_price = u32::from_be_bytes(message_data[32..36].try_into().ok()?);

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::Trade(TradeEvent {
                        quantity: executed_shares.into(),
                        price: execution_price.into(),
                        aggressor_side: OrderSide::Ask,
                        maker_id,
                    }),
                })
            }
            ITCH_MESSAGE_TYPE_ORDER_REPLACE => {
                if message_data.len() < 35 {
                    return None;
                }

                let timestamp = decode_u48(message_data[5..11].try_into().ok()?);
                let original_order_ref = u64::from_be_bytes(message_data[11..19].try_into().ok()?);
                let new_order_ref = u64::from_be_bytes(message_data[19..27].try_into().ok()?);
                let shares = u32::from_be_bytes(message_data[27..31].try_into().ok()?);
                let price = u32::from_be_bytes(message_data[31..35].try_into().ok()?);

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::L3(L3Event {
                        order_id: new_order_ref,
                        side: OrderSide::Ask,
                        timestamp,
                        kind: OrderType::Update {
                            old_id: original_order_ref,
                            qty: shares.into(),
                            price: price.into(),
                        },
                        extra: L3EventExtra::None,
                    }),
                })
            }
            ITCH_MESSAGE_TYPE_ORDER_CANCEL => {
                if message_data.len() < 23 {
                    return None;
                }

                let timestamp = decode_u48(message_data[5..11].try_into().ok()?);
                let order_reference_number =
                    u64::from_be_bytes(message_data[11..19].try_into().ok()?);
                let old_order_qty = u32::from_be_bytes(message_data[19..23].try_into().ok()?);

                Some(MarketEvent {
                    timestamp,
                    kind: MarketEventType::L3(L3Event {
                        order_id: order_reference_number,
                        side: OrderSide::Ask,
                        timestamp,
                        kind: OrderType::Cancel,
                        extra: L3EventExtra::Cancel(old_order_qty as u64), // TODO: Need to update all byte sizes to be consistent between modules
                    }),
                })
            }
            _ => None,
        }
    }

    /// Pushes a parsed market event into the output ring buffer.
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
