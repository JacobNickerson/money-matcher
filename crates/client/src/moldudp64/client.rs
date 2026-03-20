use crate::moldudp64::receiverhandler::ReceiverHandler;
use core::lob_core::market_events::MarketEvent;
use ringbuf::{HeapCons, HeapProd, HeapRb, traits::Split};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    thread,
};

pub struct MoldClient {
    pub l3_rx: HeapCons<MarketEvent>,
    pub trade_rx: HeapCons<MarketEvent>,
}

impl MoldClient {
    pub fn start() -> Self {
        let (l3_tx, l3_rx) = HeapRb::<MarketEvent>::new(2048).split();
        let (trade_tx, trade_rx) = HeapRb::<MarketEvent>::new(2048).split();

        Self::start_receiver("233.100.10.3:9503".parse().unwrap(), l3_tx);
        Self::start_receiver("233.100.10.4:9504".parse().unwrap(), trade_tx);

        Self { l3_rx, trade_rx }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::lob_core::market_events::MarketEventType;
    use ringbuf::traits::Consumer;

    #[test]
    #[ignore]
    fn receive_orders() {
        let mut mold_client = MoldClient::start();
        std::thread::sleep(std::time::Duration::from_millis(250));
        for _ in 0..50 {
            println!("");
        }

        loop {
            if let Some(event) = mold_client.l3_rx.try_pop() {
                match event.kind {
                    MarketEventType::L3(_s) => {
                        println!("Received {:?}", event);
                    }
                    _ => {
                        println!("received something..")
                    }
                }
            }

            if let Some(event) = mold_client.trade_rx.try_pop() {
                match event.kind {
                    MarketEventType::Trade(_s) => {
                        println!("Received {:?}", event);
                    }
                    _ => {
                        println!("received something..")
                    }
                }
            }
        }
    }
}
