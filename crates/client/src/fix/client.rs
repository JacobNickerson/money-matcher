use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mio::{Events, Interest, Poll, Token, Waker, event::Event, net::TcpStream};
use ringbuf::{
    HeapCons, HeapProd,
    traits::{Consumer, Producer, Split},
};

use core::fix_core::{
    messages::{
        EngineMessage, FIXEvent, FIXPayload, heartbeat::Heartbeat, logon::Logon,
        resend_request::ResendRequest, test_request::TestRequest, types::EncryptMethod,
    },
    session::{Session, SessionState},
};

const WAKE: Token = Token(0);
const SESSION: Token = Token(1);

pub struct FixClient {
    session: Option<Session>,
    server_addr: SocketAddr,
    comp_id: String,
    target_comp_id: String,
    heart_bt_int: u16,
    encrypt_method: EncryptMethod,
    outbound_rx: HeapCons<FIXEvent>,
    lob_tx: HeapProd<FIXEvent>,
    waker: Arc<Waker>,
    poll: Poll,
    poll_events: Vec<FIXEvent>,
}

impl FixClient {
    pub fn new(
        server_addr: SocketAddr,
        comp_id: String,
        target_comp_id: String,
        heart_bt_int: u16,
        encrypt_method: EncryptMethod,
        lob_tx: HeapProd<FIXEvent>,
    ) -> io::Result<(Self, FixClientHandler)> {
        let poll = Poll::new()?;
        let waker = Arc::new(Waker::new(poll.registry(), WAKE)?);

        let (outbound_tx, outbound_rx) = ringbuf::HeapRb::<FIXEvent>::new(1024).split();

        let handler = FixClientHandler {
            comp_id: comp_id.clone(),
            outbound_tx,
            waker: waker.clone(),
        };

        let client = Self {
            session: None,
            server_addr,
            comp_id,
            target_comp_id,
            heart_bt_int,
            encrypt_method,
            outbound_rx,
            lob_tx,
            waker,
            poll,
            poll_events: Vec::new(),
        };

        Ok((client, handler))
    }

    pub fn get_waker(&self) -> Arc<Waker> {
        self.waker.clone()
    }

    pub fn connect(&mut self) -> io::Result<()> {
        if self.session.is_some() {
            return Ok(());
        }

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

        let logon = Logon {
            encrypt_method: self.encrypt_method,
            heart_bt_int: self.heart_bt_int,
        };
        session
            .send_message(FIXPayload::Engine(EngineMessage::Logon(logon)), None, false)
            .ok();

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
        let mut action = None;

        if let Some(session) = self.session.as_mut() {
            let Some(state) = &session.state else { return };
            let interval = Duration::from_secs(state.heart_bt_int as u64);

            if now - session.last_received > interval {
                if session.pending_test_req.is_none() {
                    session.test_req_counter += 1;
                    session.pending_test_req = Some(session.test_req_counter);
                    action = Some(FIXPayload::Engine(EngineMessage::TestRequest(
                        TestRequest {
                            test_req_id: session.test_req_counter,
                        },
                    )));
                } else if now - session.last_received > interval + Duration::from_secs(30) {
                    self.close_session();
                    return;
                }
            } else if now - session.last_sent > interval {
                action = Some(FIXPayload::Engine(EngineMessage::Heartbeat(Heartbeat {
                    test_req_id: None,
                })));
            }
        }

        if let Some(msg) = action {
            self.send_outbound_message(msg);
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event.token() {
            WAKE => self.process_outbound_messages(),
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

    fn process_outbound_messages(&mut self) {
        while let Some(req) = self.outbound_rx.try_pop() {
            let Some(session) = self.session.as_mut() else {
                continue;
            };

            let was_empty = session.write_buffer.is_empty();
            session.send_message(req.payload, None, false).ok();
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
            if session.flush().is_err() {
                self.close_session();
                return;
            }

            if session.write_buffer.is_empty() {
                self.poll
                    .registry()
                    .reregister(&mut session.stream, SESSION, Interest::READABLE)
                    .unwrap();
                self.process_outbound_messages();
            }
        }
    }

    fn handle_readable(&mut self) {
        self.poll_events.clear();

        let result = match self.session.as_mut() {
            Some(session) => session.poll(&mut self.poll_events, &mut self.lob_tx),
            None => return,
        };

        if result.is_err() {
            self.close_session();
            return;
        }

        let events = std::mem::take(&mut self.poll_events);

        for event in events {
            match event.payload {
                FIXPayload::Engine(EngineMessage::Logon(logon)) => {
                    self.finalize_logon(event.comp_id, logon)
                }
                FIXPayload::Engine(EngineMessage::ResendRequest(resend_request)) => {
                    self.resend_messages(&resend_request);
                }
                FIXPayload::Engine(EngineMessage::TestRequest(ref test_request)) => {
                    let heartbeat = Heartbeat {
                        test_req_id: Some(test_request.test_req_id),
                    };
                    self.send_outbound_message(FIXPayload::Engine(EngineMessage::Heartbeat(
                        heartbeat,
                    )));
                }
                FIXPayload::Engine(EngineMessage::Heartbeat(ref heartbeat)) => {
                    if let Some(session) = self.session.as_mut()
                        && let Some(sent_id) = session.pending_test_req
                        && heartbeat.test_req_id == Some(sent_id)
                    {
                        session.pending_test_req = None;
                    }
                }
                _ => {
                    self.lob_tx.try_push(event).ok();
                }
            }
        }
    }

    fn close_session(&mut self) {
        if let Some(mut session) = self.session.take() {
            self.poll.registry().deregister(&mut session.stream).ok();
        }
    }

    fn resend_messages(&mut self, resend_request: &ResendRequest) {
        let mut messages_to_resend = Vec::new();

        if let Some(session) = self.session.as_mut() {
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
                        SESSION,
                        Interest::READABLE | Interest::WRITABLE,
                    )
                    .unwrap();
            }
            self.handle_writable();
        }
    }

    fn finalize_logon(&mut self, _comp_id: String, logon: Logon) {
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

        state.logged_in = true;
        state.encrypt_method = logon.encrypt_method;
        state.heart_bt_int = logon.heart_bt_int;
    }

    fn send_outbound_message(&mut self, payload: FIXPayload) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let was_empty = session.write_buffer.is_empty();
        session.send_message(payload, None, false).ok();
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

pub struct FixClientHandler {
    comp_id: String,
    outbound_tx: HeapProd<FIXEvent>,
    waker: Arc<Waker>,
}

impl FixClientHandler {
    pub fn send_message(&mut self, payload: FIXPayload) {
        let event = FIXEvent {
            comp_id: self.comp_id.clone(),
            payload,
        };

        self.outbound_tx.try_push(event).ok();
        self.waker.wake().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fix_core::messages::{
        BusinessMessage, ReportMessage,
        new_order_single::NewOrderSingle,
        types::{CustomerOrFirm, OpenClose, OrdType, PutOrCall, Side},
    };
    use ringbuf::{HeapRb, traits::Split};
    use std::thread;

    #[test]
    #[ignore]
    fn fix_client_test() {
        for _ in 0..50 {
            println!("");
        }
        let (lob_tx, mut lob_rx) = HeapRb::<FIXEvent>::new(256).split();

        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let (mut client, mut handler) = FixClient::new(
            addr,
            "CLIENT01".to_string(),
            "ENGINE01".to_string(),
            10,
            EncryptMethod::None,
            lob_tx,
        )
        .unwrap();

        client.connect().unwrap();

        let client_thread = thread::spawn(move || {
            client.run();
        });

        std::thread::sleep(Duration::from_secs(2));

        let order = NewOrderSingle {
            cl_ord_id: 1,
            handl_inst: 1,
            qty: 10,
            ord_type: OrdType::Limit,
            price: 666,
            side: Side::Buy,
            symbol: "OSISTRING".to_string(),
            open_close: OpenClose::Open,
            security_type: "OPT".to_string(),
            put_or_call: PutOrCall::Call,
            strike_price: 10,
            customer_or_firm: CustomerOrFirm::Customer,
            maturity_day: 10,
        };

        handler.send_message(FIXPayload::Business(BusinessMessage::NewOrderSingle(
            (order),
        )));

        loop {
            if let Some(cmd) = lob_rx.try_pop() {
                match cmd.payload {
                    FIXPayload::Report(msg) => match msg {
                        ReportMessage::ExecutionReport(r) => {
                            println!("Read ExecutionReport | {:?} |", r);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        client_thread.join().unwrap();
    }
}
