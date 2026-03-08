use crate::fix::session::{Session, SessionState};
use mio::event::Event;
use mio::net::TcpStream;
use mio::{Events, Interest, Poll, Token, Waker};
use netlib::fix_core::messages::heartbeat::Heartbeat;
use netlib::fix_core::messages::logon::Logon;
use netlib::fix_core::messages::test_request::TestRequest;
use netlib::fix_core::messages::types::EncryptMethod;
use netlib::fix_core::messages::{FIXReply, FIXReplyMessage, FixMessage};
use ringbuf::{HeapCons, HeapProd, traits::*};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

const WAKE: Token = Token(0);
const SESSION: Token = Token(1);

pub struct FixClient {
    session: Option<Session>,
    server_addr: SocketAddr,
    comp_id: String,
    target_comp_id: String,
    heart_bt_int: u16,
    encrypt_method: EncryptMethod,
    outbound_rx: HeapCons<Vec<u8>>,
    reply_tx: HeapProd<FIXReply>,
    waker: Arc<Waker>,
    poll: Poll,
    poll_events: Vec<FIXReply>,
}

impl FixClient {
    pub fn new(
        server_addr: SocketAddr,
        comp_id: String,
        target_comp_id: String,
        heart_bt_int: u16,
        encrypt_method: EncryptMethod,
        outbound_rx: HeapCons<Vec<u8>>,
        reply_tx: HeapProd<FIXReply>,
    ) -> io::Result<Self> {
        let poll = Poll::new()?;
        let waker = Arc::new(Waker::new(poll.registry(), WAKE)?);

        Ok(Self {
            session: None,
            server_addr,
            comp_id,
            target_comp_id,
            heart_bt_int,
            encrypt_method,
            outbound_rx,
            reply_tx,
            waker,
            poll,
            poll_events: Vec::new(),
        })
    }

    pub fn get_waker(&self) -> Arc<Waker> {
        self.waker.clone()
    }

    pub fn connect(&mut self) -> io::Result<()> {
        let mut stream = TcpStream::connect(self.server_addr)?;
        self.poll.registry().register(
            &mut stream,
            SESSION,
            Interest::READABLE | Interest::WRITABLE,
        )?;

        let mut session = Session::new(SESSION, stream);
        session.state = Some(SessionState {
            comp_id: self.comp_id.clone(),
            target_comp_id: self.target_comp_id.clone(),
            heart_bt_int: self.heart_bt_int,
            encrypt_method: self.encrypt_method,
            ..Default::default()
        });

        let logon = Logon::new(self.encrypt_method, self.heart_bt_int);
        session.handle_request(logon).ok();

        self.session = Some(session);
        Ok(())
    }

    pub fn run(&mut self) {
        let mut events = Events::with_capacity(1024);
        println!("Client connecting to {}", self.server_addr);

        loop {
            self.poll
                .poll(&mut events, Some(Duration::from_secs(1)))
                .unwrap();

            for event in events.iter() {
                self.handle_event(event);
            }

            self.check_heartbeats();
        }
    }

    pub fn check_heartbeats(&mut self) {
        let now = Instant::now();
        let mut should_close = false;
        let mut should_send_test_req = false;
        let mut should_send_heartbeat = false;
        let mut test_req_id = 0;

        if let Some(session) = self.session.as_mut() {
            let Some(state) = &session.state else {
                return;
            };

            let interval = Duration::from_secs(state.heart_bt_int as u64);

            if now - session.last_received > interval {
                if session.pending_test_req.is_none() {
                    session.test_req_counter += 1;
                    session.pending_test_req = Some(session.test_req_counter);
                    test_req_id = session.test_req_counter;
                    should_send_test_req = true;
                } else if now - session.last_received > interval + Duration::from_secs(10) {
                    should_close = true;
                }
            }

            if !should_close && !should_send_test_req && now - session.last_sent > interval {
                should_send_heartbeat = true;
            }
        }

        if should_close {
            println!("CLOSING SESSION");
            self.close_session();
        } else if should_send_test_req {
            let test_request = TestRequest { test_req_id };
            println!("Sending Test Request | {:?}", test_request);
            self.send_request(test_request);
        } else if should_send_heartbeat {
            let heartbeat = Heartbeat { test_req_id: None };
            println!("Sending heartbeat | {:?}", heartbeat);
            self.send_request(heartbeat);
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event.token() {
            WAKE => self.process_requests(),
            SESSION => {
                if event.is_writable() {
                    self.handle_writable();
                }
                if event.is_readable() {
                    self.handle_readable();
                }
            }
            _ => (),
        }
    }

    fn process_requests(&mut self) {
        while let Some(msg) = self.outbound_rx.try_pop() {
            let Some(session) = self.session.as_mut() else {
                continue;
            };

            if let Some(state) = &session.state {
                if !state.logged_in {
                    continue;
                }
            }

            let was_empty = session.write_buffer.is_empty();
            session.write_buffer.extend(msg);
            if was_empty && !session.write_buffer.is_empty() {
                self.poll
                    .registry()
                    .reregister(
                        &mut session.stream,
                        SESSION,
                        Interest::READABLE | Interest::WRITABLE,
                    )
                    .unwrap();
            }
        }
    }

    fn handle_writable(&mut self) {
        if let Some(session) = self.session.as_mut() {
            println!("Sending to server");

            if session.send_requests().is_err() {
                self.close_session();
                return;
            }

            if session.write_buffer.is_empty() {
                self.poll
                    .registry()
                    .reregister(&mut session.stream, SESSION, Interest::READABLE)
                    .unwrap();
                self.process_requests();
            }
        }
    }

    fn handle_readable(&mut self) {
        self.poll_events.clear();

        let result = match self.session.as_mut() {
            Some(session) => session.poll(&mut self.poll_events),
            None => return,
        };

        if result.is_err() {
            self.close_session();
            return;
        }

        let events = std::mem::take(&mut self.poll_events);

        for event in events {
            match event.message {
                FIXReplyMessage::Logon(logon) => self.finalize_logon(event.comp_id, logon),
                FIXReplyMessage::TestRequest(ref test_request) => {
                    let heartbeat = Heartbeat {
                        test_req_id: Some(test_request.test_req_id),
                    };
                    println!("Sending test request reply | {:?}", heartbeat);
                    self.send_request(heartbeat);
                }
                FIXReplyMessage::Heartbeat(ref heartbeat) => {
                    if let Some(session) = self.session.as_mut() {
                        if let Some(sent_id) = session.pending_test_req {
                            if heartbeat.test_req_id == Some(sent_id) {
                                println!("Clearing pending test_req");
                                session.pending_test_req = None;
                            }
                        }
                    }
                }
                _ => {
                    if let Some(session) = self.session.as_mut() {
                        if session.state.is_some() {
                            self.reply_tx.try_push(event).ok();
                        } else {
                            self.close_session();
                            return;
                        }
                    }
                }
            }
        }
    }

    fn close_session(&mut self) {
        if let Some(mut session) = self.session.take() {
            self.poll.registry().deregister(&mut session.stream).ok();
        }
    }

    fn finalize_logon(&mut self, comp_id: String, logon: Logon) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let Some(state) = session.state.as_mut() else {
            return;
        };

        if state.logged_in {
            self.close_session();
            return;
        }
        println!(
            "Finalize Logon | incoming heart_bt_int: {}",
            logon.heart_bt_int
        );

        state.logged_in = true;
        state.encrypt_method = logon.encrypt_method;
        state.heart_bt_int = logon.heart_bt_int;
    }

    fn send_request<T>(&mut self, message: T)
    where
        T: FixMessage,
    {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let was_empty = session.write_buffer.is_empty();
        session.handle_request(message).ok();
        if was_empty && !session.write_buffer.is_empty() {
            self.poll
                .registry()
                .reregister(
                    &mut session.stream,
                    SESSION,
                    Interest::READABLE | Interest::WRITABLE,
                )
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netlib::fix_core::messages::{
        new_order::NewOrder,
        types::{OpenClose, OrdType, Side},
    };
    use ringbuf::HeapRb;
    use std::thread;

    #[test]
    #[ignore]
    fn mpsc_test() {
        let (mut outbound_prod, outbound_cons) = HeapRb::<Vec<u8>>::new(256).split();
        let (reply_prod, mut reply_cons) = HeapRb::<FIXReply>::new(256).split();

        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let mut client = FixClient::new(
            addr,
            "CLIENT01".to_string(),
            "ENGINE01".to_string(),
            5,
            EncryptMethod::None,
            outbound_cons,
            reply_prod,
        )
        .unwrap();

        client.connect().unwrap();
        let waker = client.get_waker().clone();

        let client_thread = thread::spawn(move || {
            client.run();
        });

        std::thread::sleep(Duration::from_millis(100));

        loop {
            if let Some(cmd) = reply_cons.try_pop() {
                match cmd.message {
                    FIXReplyMessage::ExecutionReport(r) => {
                        println!("Read ExecutionReport | {:?} |", r);
                    }
                    FIXReplyMessage::Logon(l) => {
                        println!("Read Logon | {:?} |", l);
                    }
                    FIXReplyMessage::Heartbeat(h) => {
                        println!("Read Heartbeat | {:?} |", h);
                    }
                    FIXReplyMessage::TestRequest(tr) => {
                        println!("Read TestRequest | {:?} |", tr);
                    }
                    _ => {}
                }
            }
        }

        client_thread.join().unwrap();
    }
}
