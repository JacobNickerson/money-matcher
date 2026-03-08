use crate::fix::session::SessionState;
use crate::fix::{FIXRequest, FIXRequestMessage};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token, Waker};
use netlib::fix_core::messages::heartbeat::Heartbeat;
use netlib::fix_core::messages::logon::Logon;
use netlib::fix_core::messages::test_request::TestRequest;
use netlib::fix_core::messages::{FIXReply, FIXReplyMessage};
use ringbuf::traits::{Consumer, Producer};
use std::collections::HashMap;
use std::io::{self};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::fix::session::Session;

const LISTENER: Token = Token(0);
const WAKE: Token = Token(1);

pub struct FixEngine {
    connections: HashMap<Token, Session>,
    sessions: HashMap<String, (Token, SessionState)>,
    listener: TcpListener,
    lob_tx: ringbuf::HeapProd<FIXRequest>,
    reply_rx: ringbuf::HeapCons<FIXReply>,
    waker: Arc<Waker>,
    poll: Poll,
    token_counter: usize,
    tmp_pending_heartbeats: Vec<(Token, FIXReplyMessage)>,
    tmp_pending_close: Vec<Token>,
    poll_events: Vec<FIXRequest>,
}

impl FixEngine {
    pub fn new(
        addr: SocketAddr,
        lob_tx: ringbuf::HeapProd<FIXRequest>,
        reply_rx: ringbuf::HeapCons<FIXReply>,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let poll = Poll::new()?;
        let waker = Arc::new(Waker::new(poll.registry(), WAKE)?);
        let mut this = Self {
            connections: HashMap::new(),
            sessions: HashMap::new(),
            listener,
            lob_tx,
            reply_rx,
            waker,
            poll,
            token_counter: 100,
            tmp_pending_heartbeats: Vec::new(),
            tmp_pending_close: Vec::new(),
            poll_events: Vec::new(),
        };

        this.poll
            .registry()
            .register(&mut this.listener, LISTENER, Interest::READABLE)?;

        Ok(this)
    }

    pub fn get_waker(&self) -> Arc<Waker> {
        self.waker.clone()
    }

    pub fn run(&mut self) {
        let mut events = Events::with_capacity(1024);
        println!("Server running on {}", self.listener.local_addr().unwrap());

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

        for (token, session) in &mut self.connections {
            let Some(state) = &session.state else {
                continue;
            };

            let interval = Duration::from_secs(state.heart_bt_int as u64);

            if now - session.last_received > interval {
                if session.pending_test_req.is_none() {
                    session.test_req_counter += 1;
                    session.pending_test_req = Some(session.test_req_counter);

                    let test_request = TestRequest {
                        test_req_id: session.test_req_counter,
                    };

                    self.tmp_pending_heartbeats
                        .push((*token, FIXReplyMessage::TestRequest(test_request)));
                } else if now - session.last_received > interval + Duration::from_secs(10) {
                    self.tmp_pending_close.push(*token);
                }
            } else if now - session.last_sent > interval {
                let heartbeat = Heartbeat { test_req_id: None };
                self.tmp_pending_heartbeats
                    .push((*token, FIXReplyMessage::Heartbeat(heartbeat)));
            }
        }

        let tmp_pending_close = std::mem::take(&mut self.tmp_pending_close);
        for token in tmp_pending_close {
            self.close_session(token);
        }

        let tmp_pending_heartbeats = std::mem::take(&mut self.tmp_pending_heartbeats);
        for (token, msg) in tmp_pending_heartbeats {
            self.send_to_session(token, msg);
        }
    }

    pub fn send_reply(&mut self, reply: FIXReply) {
        let Some((token, _)) = self.sessions.get(&reply.comp_id) else {
            return;
        };
        let token = *token;
        self.send_to_session(token, reply.message);
    }

    fn send_to_session(&mut self, token: Token, msg: FIXReplyMessage) {
        let Some(session) = self.connections.get_mut(&token) else {
            return;
        };

        let was_empty = session.write_buffer.is_empty();
        session.handle_reply(msg).ok();
        if was_empty && !session.write_buffer.is_empty() {
            self.poll
                .registry()
                .reregister(
                    &mut session.stream,
                    token,
                    Interest::READABLE | Interest::WRITABLE,
                )
                .unwrap();
        }
        self.handle_writable(token);
    }

    fn handle_event(&mut self, event: &Event) {
        match event.token() {
            LISTENER => self.handle_server_accept(),
            WAKE => self.process_replies(),
            token => {
                if event.is_writable() {
                    self.handle_writable(token);
                }
                if event.is_readable() {
                    self.handle_readable(token);
                }
            }
        }
    }

    fn handle_server_accept(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((new_stream, _)) => {
                    self.register_session(new_stream).unwrap();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    break;
                }
            }
        }
    }

    fn register_session(&mut self, mut stream: TcpStream) -> io::Result<()> {
        self.poll.registry().register(
            &mut stream,
            Token(self.token_counter),
            Interest::READABLE,
        )?;

        self.connections.insert(
            Token(self.token_counter),
            Session::new(Token(self.token_counter), stream),
        );

        self.token_counter += 1;
        Ok(())
    }

    fn process_replies(&mut self) {
        while let Some(msg) = self.reply_rx.try_pop() {
            let Some((token, _)) = self.sessions.get(&msg.comp_id) else {
                continue;
            };
            let token = *token;
            self.send_to_session(token, msg.message);
        }
    }

    fn handle_writable(&mut self, token: Token) {
        if let Some(session) = self.connections.get_mut(&token) {
            if session.send_replies().is_err() {
                self.close_session(token);
                return;
            }

            if session.write_buffer.is_empty() {
                self.poll
                    .registry()
                    .reregister(&mut session.stream, token, Interest::READABLE)
                    .unwrap();
                self.process_replies();
            }
        }
    }

    fn handle_readable(&mut self, token: Token) {
        self.poll_events.clear();

        let result = match self.connections.get_mut(&token) {
            Some(session) => session.poll(&mut self.poll_events),
            None => return,
        };

        if result.is_err() {
            self.close_session(token);
            return;
        }

        let events = std::mem::take(&mut self.poll_events);

        for event in events {
            match event.message {
                FIXRequestMessage::Logon(ref logon) => {
                    self.finalize_logon(token, event.comp_id, logon)
                }
                FIXRequestMessage::TestRequest(ref test_request) => {
                    self.send_to_session(
                        token,
                        FIXReplyMessage::Heartbeat(Heartbeat {
                            test_req_id: Some(test_request.test_req_id),
                        }),
                    );
                }
                FIXRequestMessage::Heartbeat(ref heartbeat) => {
                    if let Some(session) = self.connections.get_mut(&token) {
                        if let Some(sent_id) = session.pending_test_req {
                            if heartbeat.test_req_id == Some(sent_id) {
                                println!("Clearing pending test_req | {:?}", token);
                                session.pending_test_req = None;
                            }
                        }
                    }
                }

                _ => {
                    println!("Unhandled event: {:?}", event.message);
                    if let Some(session) = self.connections.get_mut(&token) {
                        if session.state.is_some() {
                            self.lob_tx.try_push(event).ok();
                        } else {
                            self.close_session(token);
                            return;
                        }
                    }
                }
            }
        }
    }

    fn close_session(&mut self, token: Token) {
        println!("Closing session: {:?}", token);
        if let Some(mut session) = self.connections.remove(&token) {
            if let Some(state) = session.state {
                if let Some((_, stored_state)) = self.sessions.get_mut(&state.comp_id) {
                    stored_state.logged_in = false;
                    stored_state.inbound_seq_num = state.inbound_seq_num;
                    stored_state.outbound_seq_num = state.outbound_seq_num;
                }
            }
            self.poll.registry().deregister(&mut session.stream).ok();
        }
    }

    fn finalize_logon(&mut self, token: Token, comp_id: String, logon: &Logon) {
        let stored = self.sessions.entry(comp_id.clone()).or_insert_with(|| {
            (
                token,
                SessionState {
                    comp_id: comp_id.clone(),
                    target_comp_id: "ENGINE01".to_string(),
                    encrypt_method: logon.encrypt_method,
                    heart_bt_int: logon.heart_bt_int,
                    ..Default::default()
                },
            )
        });

        let (stored_token, stored_state) = stored;

        if stored_state.logged_in {
            self.close_session(token);
            return;
        }

        *stored_token = token;
        stored_state.logged_in = true;
        stored_state.inbound_seq_num += 1;

        if let Some(session) = self.connections.get_mut(&token) {
            let state = session.state.insert(stored_state.clone());
            state.comp_id = comp_id.clone();
            state.logged_in = true;
        }

        let logon_confirmation = Logon::new(stored_state.encrypt_method, stored_state.heart_bt_int);
        self.send_reply(FIXReply {
            comp_id,
            message: FIXReplyMessage::Logon(logon_confirmation),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netlib::fix_core::messages::{
        execution_report::ExecutionReport,
        types::{CustomerOrFirm, ExecTransType, ExecType, OpenClose, OrdStatus, PutOrCall, Side},
    };
    use ringbuf::traits::*;
    use std::thread;

    #[test]
    #[ignore]
    fn fix_engine_test() {
        let (lob_prod, mut lob_cons) = ringbuf::HeapRb::<FIXRequest>::new(256).split();
        let (mut reply_prod, reply_cons) = ringbuf::HeapRb::<FIXReply>::new(256).split();

        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let mut engine = FixEngine::new(addr, lob_prod, reply_cons).unwrap();
        let waker = engine.get_waker();

        let engine_thread = thread::spawn(move || {
            engine.run();
        });

        loop {
            if let Some(cmd) = lob_cons.try_pop() {
                match cmd.message {
                    FIXRequestMessage::Order(order) => {
                        println!("Read Order | {:?} | {:?} |", cmd.comp_id, order);

                        let report = ExecutionReport {
                            cl_ord_id: 1,
                            cum_qty: 0,
                            exec_id: "EXEC12345".to_string(),
                            exec_trans_type: ExecTransType::New,
                            order_id: "ORDER123".to_string(),
                            order_qty: 100,
                            ord_status: OrdStatus::New,
                            security_id: "AAAA".to_string(),
                            side: Side::Buy,
                            symbol: "AAAA".to_string(),
                            open_close: OpenClose::Open,
                            exec_type: ExecType::New,
                            leaves_qty: 100,
                            security_type: "ST".to_string(),
                            put_or_call: PutOrCall::Put,
                            strike_price: 150,
                            customer_or_firm: CustomerOrFirm::Customer,
                            maturity_date: "1".to_string(),
                        };

                        reply_prod
                            .try_push(FIXReply {
                                comp_id: cmd.comp_id,
                                message: FIXReplyMessage::ExecutionReport(report),
                            })
                            .ok();
                        waker.wake().unwrap();
                    }
                    _ => {}
                }
            }
        }

        engine_thread.join().unwrap();
    }
}
