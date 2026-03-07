use crate::fix::session::{FIXReply, FIXRequest, SessionState};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token, Waker};
use ringbuf::{HeapCons, HeapProd, traits::*};
use std::collections::HashMap;
use std::io::{self};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::fix::session::Session;

const LISTENER: Token = Token(0);
const WAKE: Token = Token(1);
const MAX_BUFFER_SIZE: usize = 1024;

pub struct FixEngine {
    connections: HashMap<Token, Session>,
    sessions: HashMap<String, (Token, SessionState)>,
    listener: TcpListener,
    tx: HeapProd<FIXRequest>,
    rx: HeapCons<FIXReply>,
    waker: Arc<Waker>,
    poll: Poll,
    token_counter: usize,
}

impl FixEngine {
    pub fn new(
        addr: SocketAddr,
        tx: HeapProd<FIXRequest>,
        rx: HeapCons<FIXReply>,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let poll = Poll::new()?;
        let waker = { Arc::new(Waker::new(poll.registry(), WAKE)?) };
        let mut this = Self {
            connections: HashMap::new(),
            sessions: HashMap::new(),
            listener,
            tx,
            rx,
            waker,
            poll,
            token_counter: 100,
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
            self.poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                self.handle_event(event);
            }
        }
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
            _ => (),
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
        while let Some(msg) = self.rx.try_pop() {
            let (comp_id, reply) = match msg {
                FIXReply::ExecutionReport(c, d) => (c, d),
            };

            let Some(token) = self.sessions.get(&comp_id).map(|(t, _)| *t) else {
                continue;
            };

            if let Some(session) = self.connections.get_mut(&token) {
                let was_empty = session.tx.is_empty();

                session.handle_reply(reply);

                if was_empty && !session.tx.is_empty() {
                    self.poll
                        .registry()
                        .reregister(
                            &mut session.stream,
                            token,
                            Interest::READABLE | Interest::WRITABLE,
                        )
                        .unwrap();
                }
            }
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
        let events = match self.connections.get_mut(&token) {
            Some(session) => match session.poll() {
                Ok(e) => e,
                Err(e) => {
                    self.close_session(token);
                    return;
                }
            },
            None => return,
        };

        for event in events {
            match event {
                FIXRequest::Logon(token, comp_id) => self.finalize_logon(token, comp_id),

                _ => {
                    if let Some(session) = self.connections.get_mut(&token) {
                        if session.state.is_some() {
                            self.tx.try_push(event).ok();
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

    fn finalize_logon(&mut self, token: Token, comp_id: String) {
        let stored = self.sessions.entry(comp_id.clone()).or_insert_with(|| {
            (
                token,
                SessionState {
                    comp_id: comp_id.clone(),
                    inbound_seq_num: 0,
                    outbound_seq_num: 0,
                    logged_in: false,
                },
            )
        });

        let (stored_token, stored_state) = stored;
        *stored_token = token;

        if stored_state.logged_in {
            self.close_session(token);
            return;
        }

        stored_state.logged_in = true;

        if let Some(session) = self.connections.get_mut(&token) {
            session.state = Some(SessionState {
                comp_id: comp_id,
                inbound_seq_num: stored_state.inbound_seq_num,
                outbound_seq_num: stored_state.outbound_seq_num,
                logged_in: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use netlib::fix_core::messages::{
        execution_report::ExecutionReport,
        types::{CustomerOrFirm, ExecTransType, ExecType, OpenClose, OrdStatus, PutOrCall, Side},
    };

    use super::*;
    use crate::fix::session::FIXRequest;
    use std::thread;

    #[test]
    #[ignore]
    fn mpsc_test() {
        let (mut prod, mut cons) = ringbuf::HeapRb::<FIXRequest>::new(256).split();

        let (mut reply_prod, mut reply_cons) = ringbuf::HeapRb::<FIXReply>::new(256).split();

        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let mut engine = FixEngine::new(addr, prod, reply_cons).unwrap();
        let waker = engine.get_waker();
        let engine_thread = thread::spawn(move || {
            engine.run();
        });

        loop {
            if let Some(cmd) = cons.try_pop() {
                match cmd {
                    FIXRequest::Order(comp_id, order) => {
                        println!("Read Order | {:?} | {:?} |", comp_id, order,);

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
                            .try_push(FIXReply::ExecutionReport(comp_id, report))
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
