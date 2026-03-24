use crate::moldudp64::receiverhandler::ReceiverHandler;
use mm_core::lob_core::market_events::{L3Event, MarketEvent, TradeEvent};
use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Split},
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::Mutex,
    thread,
};

pub struct MoldClient {
    pub l3_rx: Mutex<HeapCons<MarketEvent>>,
    pub trade_rx: Mutex<HeapCons<MarketEvent>>,
    l3_cache: VecDeque<MarketEvent>,
    trade_cache: VecDeque<MarketEvent>,
}

impl MoldClient {
    pub fn start() -> Self {
        let (l3_tx, l3_rx) = HeapRb::<MarketEvent>::new(1024).split();
        let (trade_tx, trade_rx) = HeapRb::<MarketEvent>::new(1024).split();

        Self::start_receiver("233.100.10.3:9503".parse().unwrap(), l3_tx);
        Self::start_receiver("233.100.10.4:9504".parse().unwrap(), trade_tx);

        Self {
            l3_rx: Mutex::new(l3_rx),
            trade_rx: Mutex::new(trade_rx),
            l3_cache: VecDeque::new(),
            trade_cache: VecDeque::new(),
        }
    }

    fn start_receiver(addr: SocketAddr, tx: HeapProd<MarketEvent>) {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("err");

        socket.set_reuse_address(true).expect("err");

        let bind_addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port());
        socket.bind(&SockAddr::from(bind_addr)).expect("err");

        if let IpAddr::V4(ip) = addr.ip() {
            socket
                .join_multicast_v4(&ip, &Ipv4Addr::LOCALHOST)
                .expect("Failed to join");
        }

        let std_socket: UdpSocket = socket.into();
        let receiver_handler = ReceiverHandler::new(tx, std_socket);

        thread::spawn(move || {
            receiver_handler.run();
        });
    }

    pub fn next_event(&mut self) -> Option<MarketEvent> {
        if self.l3_cache.is_empty() {
            let mut cons = self.l3_rx.lock().unwrap();
            self.l3_cache.extend(cons.pop_iter());
        }

        if self.trade_cache.is_empty() {
            let mut cons = self.trade_rx.lock().unwrap();
            self.trade_cache.extend(cons.pop_iter());
        }

        match (self.l3_cache.front(), self.trade_cache.front()) {
            (Some(l3), Some(trade)) => {
                if l3.timestamp <= trade.timestamp {
                    self.l3_cache.pop_front()
                } else {
                    self.trade_cache.pop_front()
                }
            }
            (Some(_), None) => self.l3_cache.pop_front(),
            (None, Some(_)) => self.trade_cache.pop_front(),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    #[ignore]
    fn receive_orders() {
        let mut mold_client = MoldClient::start();
        std::thread::sleep(Duration::from_millis(250));
        for _ in 0..50 {
            println!("");
        }

        let mut count = 0;
        let mut last_received = Instant::now();

        loop {
            let now = Instant::now();
            if let Some(event) = mold_client.next_event() {
                //println!("Received {:?}", event);
                count = count + 1;
                last_received = now;
            } else if count > 0 && now - last_received > Duration::from_secs(5) {
                println!("\n\nReceived {} events", count);
                count = 0;
            }
        }
    }
}
