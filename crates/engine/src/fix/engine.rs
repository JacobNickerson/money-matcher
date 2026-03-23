use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token, Waker};
use ringbuf::HeapProd;
use ringbuf::traits::{Consumer, Producer, Split};

use mm_core::fix_core::{
    messages::{
        EngineMessage, FIXEvent, FIXPayload, heartbeat::Heartbeat, logon::Logon,
        resend_request::ResendRequest, test_request::TestRequest,
    },
    session::{Session, SessionState},
};

const LISTENER: Token = Token(0);
const WAKE: Token = Token(1);

pub struct FixEngine {
    comp_id: Arc<str>,
    connections: HashMap<Token, Session>,
    sessions: HashMap<Arc<str>, (Token, SessionState)>,
    listener: TcpListener,
    lob_tx: ringbuf::HeapProd<FIXEvent>,
    outbound_rx: ringbuf::HeapCons<FIXEvent>,
    waker: Arc<Waker>,
    poll: Poll,
    token_counter: usize,
    tmp_pending_heartbeats: Vec<(Token, FIXPayload)>,
    tmp_pending_close: Vec<Token>,
    poll_events: Vec<FIXEvent>,
}

impl FixEngine {
    pub fn new(
        addr: SocketAddr,
        comp_id: String,
        lob_tx: ringbuf::HeapProd<FIXEvent>,
    ) -> io::Result<(Self, FixEngineHandler)> {
        let listener = TcpListener::bind(addr)?;
        let poll = Poll::new()?;
        let waker = Arc::new(Waker::new(poll.registry(), WAKE)?);

        let (outbound_tx, outbound_rx) = ringbuf::HeapRb::<FIXEvent>::new(1024).split();

        let handler = FixEngineHandler {
            outbound_tx,
            waker: waker.clone(),
        };

        let mut engine = Self {
            comp_id: Arc::from(comp_id),
            connections: HashMap::new(),
            sessions: HashMap::new(),
            listener,
            lob_tx,
            outbound_rx,
            waker,
            poll,
            token_counter: 100,
            tmp_pending_heartbeats: Vec::new(),
            tmp_pending_close: Vec::new(),
            poll_events: Vec::new(),
        };

        engine
            .poll
            .registry()
            .register(&mut engine.listener, LISTENER, Interest::READABLE)?;

        Ok((engine, handler))
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

                    self.tmp_pending_heartbeats.push((
                        *token,
                        FIXPayload::Engine(EngineMessage::TestRequest(test_request)),
                    ));
                } else if now - session.last_received > interval + Duration::from_secs(30) {
                    self.tmp_pending_close.push(*token);
                }
            } else if now - session.last_sent > interval {
                let heartbeat = Heartbeat { test_req_id: None };
                self.tmp_pending_heartbeats.push((
                    *token,
                    FIXPayload::Engine(EngineMessage::Heartbeat(heartbeat)),
                ));
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

    pub fn send_outbound_message(&mut self, request: FIXEvent) {
        let Some((token, _)) = self.sessions.get(&request.comp_id) else {
            return;
        };
        let token = *token;
        self.send_to_session(token, request.payload);
    }

    fn send_to_session(&mut self, token: Token, msg: FIXPayload) {
        let Some(session) = self.connections.get_mut(&token) else {
            return;
        };

        let was_empty = session.write_buffer.is_empty();
        session.send_message(msg, None, false).ok();
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
            WAKE => self.process_outbound_messages(),
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
                Err(_e) => {
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

    fn process_outbound_messages(&mut self) {
        while let Some(msg) = self.outbound_rx.try_pop() {
            let Some((token, _)) = self.sessions.get(&msg.comp_id) else {
                continue;
            };
            let token = *token;
            self.send_to_session(token, msg.payload);
        }
    }

    fn handle_writable(&mut self, token: Token) {
        if let Some(session) = self.connections.get_mut(&token) {
            if session.flush().is_err() {
                self.close_session(token);
                return;
            }

            if session.write_buffer.is_empty() {
                self.poll
                    .registry()
                    .reregister(&mut session.stream, token, Interest::READABLE)
                    .unwrap();
                self.process_outbound_messages();
            }
        }
    }

    fn handle_readable(&mut self, token: Token) {
        self.poll_events.clear();

        let result = match self.connections.get_mut(&token) {
            Some(session) => session.poll(&mut self.poll_events, &mut self.lob_tx),
            None => return,
        };

        if result.is_err() {
            self.close_session(token);
            return;
        }

        if let Some(session) = self.connections.get_mut(&token)
            && !session.write_buffer.is_empty()
        {
            self.poll
                .registry()
                .reregister(
                    &mut session.stream,
                    token,
                    Interest::READABLE | Interest::WRITABLE,
                )
                .unwrap();
        }

        let events = std::mem::take(&mut self.poll_events);

        for event in events {
            match event.payload {
                FIXPayload::Engine(EngineMessage::Logon(ref logon)) => {
                    self.finalize_logon(token, Arc::clone(&event.comp_id), logon)
                }
                FIXPayload::Engine(EngineMessage::ResendRequest(ref resend_request)) => {
                    self.resend_messages(token, resend_request);
                }
                FIXPayload::Engine(EngineMessage::TestRequest(ref test_request)) => {
                    self.send_to_session(
                        token,
                        FIXPayload::Engine(EngineMessage::Heartbeat(Heartbeat {
                            test_req_id: Some(test_request.test_req_id),
                        })),
                    );
                }
                FIXPayload::Engine(EngineMessage::Heartbeat(ref heartbeat)) => {
                    if let Some(session) = self.connections.get_mut(&token)
                        && let Some(sent_id) = session.pending_test_req
                        && heartbeat.test_req_id == Some(sent_id)
                    {
                        session.pending_test_req = None;
                    }
                }
                _ => {
                    println!("Unhandled engine event: {:?}", event.payload);
                }
            }
        }
    }

    fn close_session(&mut self, token: Token) {
        if let Some(mut session) = self.connections.remove(&token) {
            if let Some(state) = session.state
                && let Some((_, stored_state)) = self.sessions.get_mut(&state.comp_id)
            {
                stored_state.logged_in = false;
                stored_state.inbound_seq_num = state.inbound_seq_num;
                stored_state.outbound_seq_num = state.outbound_seq_num;
            }
            self.poll.registry().deregister(&mut session.stream).ok();
        }
    }

    fn resend_messages(&mut self, token: Token, resend_request: &ResendRequest) {
        let mut messages_to_resend = Vec::new();

        if let Some(session) = self.connections.get_mut(&token) {
            if let Some(state) = &session.state {
                let end = if resend_request.end_seq_no == 0 {
                    u32::MAX
                } else {
                    resend_request.end_seq_no
                };

                for (&seq, msg) in state.sent_messages.range(resend_request.begin_seq_no..=end) {
                    messages_to_resend.push((seq, msg.clone()));
                }
            }

            let was_empty = session.write_buffer.is_empty();

            for (seq, msg) in messages_to_resend {
                session.send_message(msg.clone(), Some(seq), true).ok();
            }

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
    }

    fn finalize_logon(&mut self, token: Token, comp_id: Arc<str>, logon: &Logon) {
        let stored = self
            .sessions
            .entry(Arc::clone(&comp_id))
            .or_insert_with(|| {
                (
                    token,
                    SessionState {
                        comp_id: Arc::clone(&comp_id),
                        target_comp_id: Arc::from("ENGINE01"),
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
            state.logged_in = true;
        }

        let logon_confirmation = Logon {
            encrypt_method: stored_state.encrypt_method,
            heart_bt_int: stored_state.heart_bt_int,
        };
        self.send_outbound_message(FIXEvent {
            comp_id: Arc::clone(&comp_id),
            payload: FIXPayload::Engine(EngineMessage::Logon(logon_confirmation)),
        });
    }
}

pub struct FixEngineHandler {
    outbound_tx: HeapProd<FIXEvent>,
    waker: Arc<Waker>,
}

impl FixEngineHandler {
    pub fn send_message(&mut self, event: FIXEvent) {
        self.outbound_tx.try_push(event).ok();
        self.waker.wake().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::fix_core::messages::{
        BusinessMessage, ReportMessage,
        execution_report::ExecutionReport,
        types::{CustomerOrFirm, ExecTransType, ExecType, OpenClose, OrdStatus, PutOrCall, Side},
    };
    use std::thread;

    #[test]
    #[ignore]
    fn fix_engine_test() {
        for _ in 0..50 {
            println!("");
        }
        let (lob_tx, mut lob_rx) = ringbuf::HeapRb::<FIXEvent>::new(256).split();

        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let (mut engine, mut handler) =
            FixEngine::new(addr, "ENGINE01".to_owned(), lob_tx).unwrap();

        let engine_thread = thread::spawn(move || {
            engine.run();
        });

        loop {
            if let Some(cmd) = lob_rx.try_pop() {
                match cmd.payload {
                    FIXPayload::Business(msg) => match msg {
                        BusinessMessage::NewOrderSingle(order) => {
                            println!("Read Order | {:?} | {:?} |", cmd.comp_id, order);

                            let report = ExecutionReport {
                                cl_ord_id: order.cl_ord_id,
                                cum_qty: 0,
                                exec_id: "EXEC12345".to_string(),
                                exec_trans_type: ExecTransType::New,
                                order_id: "ORDER123".to_string(),
                                order_qty: order.qty,
                                ord_status: OrdStatus::New,
                                security_id: "SECURITYID".to_string(),
                                side: order.side,
                                symbol: order.symbol,
                                open_close: order.open_close,
                                exec_type: ExecType::New,
                                leaves_qty: order.qty,
                                security_type: order.security_type,
                                put_or_call: order.put_or_call,
                                strike_price: order.strike_price,
                                customer_or_firm: order.customer_or_firm,
                                maturity_date: "202603".to_string(),
                            };

                            handler.send_message(FIXEvent {
                                comp_id: cmd.comp_id,
                                payload: FIXPayload::Report(ReportMessage::ExecutionReport(report)),
                            });
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        engine_thread.join().unwrap();
    }
}
