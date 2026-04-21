use crate::moldudp64::receiverhandler::ReceiverHandler;
use mm_core::lob_core::market_events::MarketEvent;
use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Split},
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    thread,
};

/// A multicast client that joins L3 and Trade multicast groups to receive and merge market events.
pub struct MoldClient {
    pub l3_rx: HeapCons<MarketEvent>,
    pub trade_rx: HeapCons<MarketEvent>,
    next_l3: Option<MarketEvent>,
    next_trade: Option<MarketEvent>,
    expected_tracking_number: u16,
}

impl MoldClient {
    /// Initializes the client and spawns background threads for L3 and Trade multicast receivers.
    pub fn start() -> Self {
        let (l3_tx, l3_rx) = HeapRb::<MarketEvent>::new(1 << 24).split();
        let (trade_tx, trade_rx) = HeapRb::<MarketEvent>::new(1 << 24).split();

        Self::start_receiver(
            "233.100.10.3:9503".parse().unwrap(),
            "127.0.0.1:9003".parse().unwrap(),
            l3_tx,
        );
        Self::start_receiver(
            "233.100.10.4:9504".parse().unwrap(),
            "127.0.0.1:9004".parse().unwrap(),
            trade_tx,
        );

        Self {
            l3_rx,
            trade_rx,
            next_l3: None,
            next_trade: None,
            expected_tracking_number: 1,
        }
    }

    /// Configures a UDP socket to join a specific multicast group and spawns a `ReceiverHandler` to process incoming packets.
    fn start_receiver(
        addr: SocketAddr,
        retransmission_addr: SocketAddr,
        tx: HeapProd<MarketEvent>,
    ) {
        let multicast_socket =
            Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("err");
        multicast_socket
            .set_recv_buffer_size(32 * 1024 * 1024)
            .expect("err");
        multicast_socket.set_reuse_address(true).expect("err");

        let bind_addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port());
        multicast_socket
            .bind(&SockAddr::from(bind_addr))
            .expect("err");

        if let IpAddr::V4(ip) = addr.ip() {
            multicast_socket
                .join_multicast_v4(&ip, &Ipv4Addr::LOCALHOST)
                .expect("Failed to join");
        }

        let std_socket: UdpSocket = multicast_socket.into();

        let retransmission_socket =
            UdpSocket::bind(SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0)).expect("err");
        retransmission_socket.set_nonblocking(true).expect("err");
        let receiver_handler =
            ReceiverHandler::new(tx, std_socket, retransmission_socket, retransmission_addr);

        thread::spawn(move || {
            receiver_handler.run();
        });
    }

    /// Retrieves the next market event from the combined L3 and Trade feeds, sorted by tracking number to ensure correct processing order.
    pub fn next_event(&mut self) -> Option<MarketEvent> {
        if self.next_l3.is_none() {
            self.next_l3 = self.l3_rx.try_pop();
        }

        if self.next_trade.is_none() {
            self.next_trade = self.trade_rx.try_pop();
        }

        let event = if let Some(l3) = self.next_l3 {
            if l3.id == self.expected_tracking_number {
                self.next_l3.take()
            } else {
                None
            }
        } else {
            None
        };

        let event = if event.is_none() {
            if let Some(trade) = self.next_trade {
                if trade.id == self.expected_tracking_number {
                    self.next_trade.take()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            event
        };

        if let Some(ev) = event {
            self.expected_tracking_number = self.expected_tracking_number.wrapping_add(1);
            Some(ev)
        } else {
            None
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
                println!("Received {:?}", event);
                count = count + 1;
                last_received = now;
            } else if count > 0 && now - last_received > Duration::from_secs(10) {
                println!("\n\nReceived {} events", count);
                count = 0;
            } else {
                std::hint::spin_loop();
            }
        }
    }
}
