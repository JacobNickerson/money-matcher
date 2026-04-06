use crate::moldudp64::sequencerpublisher::SequencerPublisher;
use bytes::Bytes;
use mm_core::{
    itch_core::messages::{
        add_order::AddOrder, order_cancel::OrderCancel,
        order_executed_with_price::OrderExecutedWithPrice, order_replace::OrderReplace,
    },
    lob_core::{
        market_events::{EventSink, L3EventExtra, MarketEvent, MarketEventType},
        market_orders::OrderType,
    },
    moldudp64_core::types::Event,
};
use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Producer, Split},
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, atomic::AtomicBool},
    thread,
};

/// A multicast engine that translates internal market events into ITCH protocol messages for UDP broadcast.
pub struct MoldEngine {
    l3_tx: HeapProd<Event>,
    trade_tx: HeapProd<Event>,
    current_tracking_number: u16,
}

impl MoldEngine {
    /// Initializes the engine and spawns background threads for L3 and Trade multicast publishers.
    pub fn start(running: Arc<AtomicBool>) -> Self {
        let (l3_tx, l3_rx) = HeapRb::<Event>::new(1 << 20).split();
        let (trade_tx, trade_rx) = HeapRb::<Event>::new(1 << 20).split();

        Self::start_publisher(
            "MM_L3".to_string(),
            "233.100.10.3:9503".parse().unwrap(),
            9503,
            l3_rx,
            Arc::clone(&running),
        );
        Self::start_publisher(
            "MM_TR".to_string(),
            "233.100.10.4:9504".parse().unwrap(),
            9504,
            trade_rx,
            Arc::clone(&running),
        );

        Self {
            l3_tx,
            trade_tx,
            current_tracking_number: 1,
        }
    }

    /// Configures a UDP socket for multicast broadcasting and runs the publisher event loop.
    fn start_publisher(
        session_id: String,
        multicast_group: SocketAddr,
        retransmission_port: u16,
        event_rx: HeapCons<Event>,
        running: Arc<AtomicBool>,
    ) {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("err");
        socket.set_multicast_loop_v4(true).expect("err");

        let bind_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
        socket.bind(&SockAddr::from(bind_addr)).expect("err");
        socket.set_multicast_ttl_v4(1).expect("err");
        socket
            .set_multicast_if_v4(&Ipv4Addr::LOCALHOST)
            .expect("err");

        let std_socket: UdpSocket = socket.into();

        let retransmission_addr = format!("0.0.0.0:{}", retransmission_port);
        let retransmission_socket = UdpSocket::bind(retransmission_addr).expect("err");
        retransmission_socket.set_nonblocking(true).expect("err");

        let sequencer_publisher = SequencerPublisher::new(
            event_rx,
            multicast_group,
            std_socket,
            retransmission_socket,
            session_id,
            running,
        );

        thread::spawn(move || {
            sequencer_publisher.run();
        });
    }

    /// Queues a raw byte payload into the designated ring buffer for transmission.
    fn push_event(channel_tx: &mut HeapProd<Bytes>, buf: &[u8]) {
        let bytes = Bytes::copy_from_slice(buf);

        channel_tx.try_push(bytes).ok();
    }

    pub fn push(&mut self, event: MarketEvent) {
        match event.kind {
            MarketEventType::L3(e) => match e.kind {
                OrderType::Limit { qty, price } => {
                    let mut buf = [0u8; 36];

                    AddOrder::encode_into(
                        &mut buf,
                        0, // PLACEHOLDER
                        self.current_tracking_number,
                        event.timestamp,
                        e.order_id,
                        e.side as u8,
                        qty.try_into().unwrap(),
                        *b"  stock ", // PLACEHOLDER
                        price as u32,
                    );

                    self.current_tracking_number = self.current_tracking_number.wrapping_add(1);
                    Self::push_event(&mut self.l3_tx, &buf);
                }

                OrderType::Market { qty } => {
                    let mut buf = [0u8; 36];

                    AddOrder::encode_into(
                        &mut buf,
                        0, // PLACEHOLDER
                        self.current_tracking_number,
                        event.timestamp,
                        e.order_id,
                        e.side as u8,
                        qty.try_into().unwrap(),
                        *b"  stock ", // PLACEHOLDER
                        0u32,
                    );

                    self.current_tracking_number = self.current_tracking_number.wrapping_add(1);
                    Self::push_event(&mut self.l3_tx, &buf);
                }
                OrderType::Cancel => {
                    let mut buf = [0u8; 23];

                    let L3EventExtra::Cancel(cancel_qty) = e.extra else {
                        panic!(
                            "Expected L3EventExtra::Cancel for cancel event, but got {:?}",
                            e.extra
                        );
                    };
                    OrderCancel::encode_into(
                        &mut buf,
                        0, // PLACEHOLDER
                        self.current_tracking_number,
                        event.timestamp,
                        e.order_id,
                        cancel_qty.try_into().unwrap(), // TODO: Need to update various struct members to be the same byte size
                    );

                    self.current_tracking_number = self.current_tracking_number.wrapping_add(1);
                    Self::push_event(&mut self.l3_tx, &buf);
                }
                OrderType::Update { old_id, qty, price } => {
                    let mut buf = [0u8; 35];

                    OrderReplace::encode_into(
                        &mut buf,
                        0, // PLACEHOLDER
                        self.current_tracking_number,
                        event.timestamp,
                        old_id,
                        e.order_id,
                        qty.try_into().unwrap(),
                        price as u32,
                    );

                    self.current_tracking_number = self.current_tracking_number.wrapping_add(1);
                    Self::push_event(&mut self.l3_tx, &buf);
                }
                _ => {}
            },
            MarketEventType::Trade(e) => {
                let mut buf = [0u8; 36];

                OrderExecutedWithPrice::encode_into(
                    &mut buf,
                    0, // PLACEHOLDER
                    self.current_tracking_number,
                    event.timestamp,
                    e.maker_id,
                    e.quantity as u32,
                    0,    // PLACEHOLDER
                    b'Y', // PLACEHOLDER
                    e.price as u32,
                );

                self.current_tracking_number = self.current_tracking_number.wrapping_add(1);

                Self::push_event(&mut self.trade_tx, &buf);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::lob_core::{
        market_events::{
            EventSink, L3Event, L3EventExtra, MarketEvent, MarketEventType, TradeEvent,
        },
        market_orders::OrderSide,
    };

    #[test]
    #[ignore]
    fn send_orders() {
        let mut server = MoldEngine::start(Arc::new(AtomicBool::new(true)));
        std::thread::sleep(std::time::Duration::from_millis(250));
        for _ in 0..50 {
            println!("");
        }

        let mut i = 0;

        for _ in 0..5 {
            i = i + 1;

            let limit_event = MarketEvent {
                timestamp: i,
                kind: MarketEventType::L3(L3Event {
                    order_id: i,
                    timestamp: i,
                    side: OrderSide::Ask,
                    kind: OrderType::Limit {
                        qty: 100,
                        price: 500,
                    },
                    extra: L3EventExtra::None,
                }),
            };

            println!("Sending {:?}", limit_event);
            server.push(limit_event.clone());
        }

        for _ in 0..5 {
            i = i + 1;

            let cancel_event = MarketEvent {
                timestamp: i,
                kind: MarketEventType::L3(L3Event {
                    order_id: i,
                    timestamp: i,
                    side: OrderSide::Ask,
                    kind: OrderType::Cancel,
                    extra: L3EventExtra::Cancel(100),
                }),
            };

            println!("Sending {:?}", cancel_event);
            server.push(cancel_event.clone());
        }

        for _ in 0..5 {
            i = i + 1;

            let update_event = MarketEvent {
                timestamp: i,
                kind: MarketEventType::L3(L3Event {
                    order_id: i,
                    timestamp: i,
                    side: OrderSide::Ask,
                    kind: OrderType::Update {
                        old_id: i - 1,
                        qty: i,
                        price: i,
                    },
                    extra: L3EventExtra::None,
                }),
            };

            println!("Sending {:?}", update_event);
            server.push(update_event.clone());
        }

        for _ in 0..5 {
            i = i + 1;

            let trade_event = MarketEvent {
                kind: MarketEventType::Trade(TradeEvent {
                    price: i,
                    quantity: i,
                    aggressor_side: OrderSide::Ask,
                    maker_id: i,
                }),
                timestamp: i,
            };

            println!("Sending {:?}", trade_event);
            server.push(trade_event.clone());
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
