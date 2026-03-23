use crate::moldudp64::receiverhandler::ReceiverHandler;
use mm_core::lob_core::market_events::MarketEvent;
use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Split},
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::Mutex,
    thread,
};

pub struct MoldClient {
    pub l3_rx: Mutex<HeapCons<MarketEvent>>,
    pub trade_rx: Mutex<HeapCons<MarketEvent>>,
    next_l3: Option<MarketEvent>,
    next_trade: Option<MarketEvent>,
}

impl MoldClient {
    pub fn start() -> Self {
        let (l3_tx, l3_rx) = HeapRb::<MarketEvent>::new(2048).split();
        let (trade_tx, trade_rx) = HeapRb::<MarketEvent>::new(2048).split();

        Self::start_receiver("233.100.10.3:9503".parse().unwrap(), l3_tx);
        Self::start_receiver("233.100.10.4:9504".parse().unwrap(), trade_tx);

        Self {
            l3_rx: Mutex::new(l3_rx),
            trade_rx: Mutex::new(trade_rx),
            next_l3: None,
            next_trade: None,
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
        if self.next_l3.is_none() {
            self.next_l3 = self.l3_rx.lock().unwrap().try_pop();
        }
        if self.next_trade.is_none() {
            self.next_trade = self.trade_rx.lock().unwrap().try_pop();
        }

        match (&self.next_l3, &self.next_trade) {
            (Some(l3), Some(trade)) => {
                if l3.timestamp <= trade.timestamp {
                    self.next_l3.take()
                } else {
                    self.next_trade.take()
                }
            }
            (Some(_), None) => self.next_l3.take(),
            (None, Some(_)) => self.next_trade.take(),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn receive_orders() {
        let mut mold_client = MoldClient::start();
        std::thread::sleep(std::time::Duration::from_millis(250));
        for _ in 0..50 {
            println!("");
        }

        loop {
            if let Some(event) = mold_client.next_event() {
                println!("Received {:?}", event);
            }
        }
    }
}
