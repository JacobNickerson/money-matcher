use crate::fix::session::Session;
use mio::{Poll, Token, Waker};
use netlib::fix_core::messages::{FixFrame, FixMessage};
use ringbuf::traits::Producer;
use ringbuf::{HeapCons, HeapProd};
use std::io;
use std::net::SocketAddr;

pub struct FixClient {
    session_tx: HeapProd<FixFrame>,
    waker: Waker,
}

const WAKE: Token = Token(1);
impl FixClient {
    pub fn start(
        session_tx: HeapProd<FixFrame>,
        session_rx: HeapCons<FixFrame>,
    ) -> io::Result<(Self, Session)> {
        let poll = Poll::new()?;
        let waker = { Waker::new(poll.registry(), WAKE)? };

        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let session = Session::connect(addr, poll, session_rx)?;

        let this = Self { session_tx, waker };

        Ok((this, session))
    }

    pub fn push_command<T>(&mut self, cmd: T) -> Result<(), &'static str>
    where
        T: FixMessage,
    {
        let frame = FixFrame {
            msg_type: T::MESSAGE_TYPE,
            body: cmd.as_bytes(),
        };

        self.session_tx.try_push(frame).map_err(|_| "queue full")?;

        self.waker.wake().map_err(|_| "wake error")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netlib::fix_core::messages::{
        new_order::NewOrder,
        types::{OpenClose, OrdType, Side},
    };
    use ringbuf::traits::Split;
    use std::thread;

    #[test]
    #[ignore]
    fn test() {
        let (mut session_tx, mut session_rx) = ringbuf::HeapRb::<FixFrame>::new(256).split();

        let (mut client, mut session) = FixClient::start(session_tx, session_rx).unwrap();

        let client_thread = thread::spawn(move || {
            let mut ses = session;
            ses.run();
        });

        let _ = client.push_command(NewOrder::new(
            1,
            1,
            10,
            OrdType::Limit,
            666,
            Side::Buy,
            "OSISTRING".to_string(),
            OpenClose::Open,
            "OPT".to_string(),
        ));

        client_thread.join().unwrap();
    }
}
