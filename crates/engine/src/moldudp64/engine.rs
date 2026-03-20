use bytes::Bytes;
use core::lob_core::{
    OrderId, Price, Timestamp,
    market_events::{
        ClientEvent, ClientEventType, EventSink, L1Event, L2Event, L3Event, LiquidityFlag,
        MarketEvent, MarketEventType, TradeEvent,
    },
    market_orders::{LimitOrder, Order, OrderSide, OrderStatus, OrderType},
};
use core::{
    itch_core::messages::{add_order::AddOrder, order_executed_with_price::OrderExecutedWithPrice},
    moldudp64_core::types::Event,
};
use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Producer, Split},
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    thread,
};

use crate::moldudp64::sequencerpublisher::SequencerPublisher;

pub struct MoldEngine {
    l3_tx: HeapProd<Event>,
    trade_tx: HeapProd<Event>,
    current_tracking_number: u16,
}

impl MoldEngine {
    pub fn start() -> Self {
        let (l3_tx, l3_rx) = HeapRb::<Event>::new(2048).split();
        let (trade_tx, trade_rx) = HeapRb::<Event>::new(2048).split();

        Self::start_publisher(
            "MM_L3".to_string(),
            "233.100.10.3:9503".parse().unwrap(),
            l3_rx,
        );
        Self::start_publisher(
            "MM_TR".to_string(),
            "233.100.10.4:9504".parse().unwrap(),
            trade_rx,
        );

        Self {
            l3_tx,
            trade_tx,
            current_tracking_number: 1,
        }
    }

    fn start_publisher(session_id: String, multicast_group: SocketAddr, event_rx: HeapCons<Event>) {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("err");
        socket.set_multicast_loop_v4(true).expect("err");

        let bind_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
        socket.bind(&SockAddr::from(bind_addr)).expect("err");
        socket.set_multicast_ttl_v4(1).expect("err");
        socket
            .set_multicast_if_v4(&Ipv4Addr::LOCALHOST)
            .expect("err");

        let std_socket: UdpSocket = socket.into();

        let sequencer_publisher =
            SequencerPublisher::new(event_rx, multicast_group, std_socket, session_id);

        thread::spawn(move || {
            sequencer_publisher.run();
        });
    }

    pub fn push_event(channel_tx: &mut HeapProd<Bytes>, buf: &[u8]) {
        let bytes = Bytes::copy_from_slice(buf);

        channel_tx.try_push(bytes).ok();
    }
}

impl EventSink for MoldEngine {
    fn push(&mut self, event: MarketEvent) {
        match event.kind {
            MarketEventType::L1(_e) => {}
            MarketEventType::L2(_e) => {}
            MarketEventType::L3(e) => {
                let mut buf = [0u8; 36];

                AddOrder::encode_into(
                    &mut buf,
                    0,
                    self.current_tracking_number,
                    event.timestamp,
                    e.order_id,
                    e.side as u8,
                    e.qty.try_into().unwrap(),
                    *b"   stock",
                    e.price as u32,
                );

                self.current_tracking_number = self.current_tracking_number.wrapping_add(1);

                Self::push_event(&mut self.l3_tx, &buf);
            }
            MarketEventType::Trade(e) => {
                let mut buf = [0u8; 36];

                OrderExecutedWithPrice::encode_into(
                    &mut buf,
                    0,
                    self.current_tracking_number,
                    event.timestamp,
                    0,
                    e.quantity as u32,
                    0,
                    b'Y',
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
    use core::lob_core::{
        market_events::{EventSink, L3Event, MarketEvent, MarketEventType, TradeEvent},
        market_orders::{OrderSide, OrderStatus},
    };

    #[test]
    #[ignore]
    fn send_orders() {
        let mut server = MoldEngine::start();
        std::thread::sleep(std::time::Duration::from_millis(250));
        for _ in 0..50 {
            println!("");
        }

        let mut i = 0;

        for _ in 0..5 {
            i = i + 1;

            let l3_event = MarketEvent {
                kind: MarketEventType::L3(L3Event {
                    order_id: i,
                    side: OrderSide::Ask,
                    qty: 100,
                    price: 500,
                    status: OrderStatus::Active,
                }),
                timestamp: i,
            };

            println!("Sending {:?}", l3_event);
            server.push(l3_event.clone());
        }

        for _ in 0..5 {
            i = i + 1;

            let trade_event = MarketEvent {
                kind: MarketEventType::Trade(TradeEvent {
                    price: 500,
                    quantity: 500,
                    aggressor_side: OrderSide::Ask,
                }),
                timestamp: i,
            };

            println!("Sending {:?}", trade_event);
            server.push(trade_event.clone());
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
