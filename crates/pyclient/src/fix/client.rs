use mio::{Events, Interest, Poll, Token, Waker, event::Event, net::TcpStream};
use mm_core::{
    fix_core::{
        messages::{
            BusinessMessage, EngineMessage, FIXBusinessMessage, FIXEvent, FIXPayload,
            heartbeat::Heartbeat, logon::Logon, resend_request::ResendRequest,
            test_request::TestRequest, types::EncryptMethod,
        },
        session::{Session, SessionState},
    },
    lob_core::market_orders::Order,
};
use ringbuf::{
    HeapCons, HeapProd,
    traits::{Consumer, Producer, Split},
};
use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Token used to interrupt the poll loop when the application queues new outbound messages.
const WAKE: Token = Token(0);
/// Token identifying the main FIX TCP stream.
const SESSION: Token = Token(1);

/// A FIX client that manages the TCP connection, session state, and network event loop.
pub struct FixClient {
    session: Option<Session>,
    server_addr: SocketAddr,
    comp_id: Arc<str>,
    target_comp_id: Arc<str>,
    heart_bt_int: u16,
    encrypt_method: EncryptMethod,
    outbound_rx: HeapCons<FIXEvent>,
    lob_tx: HeapProd<FIXEvent>,
    poll: Poll,
    poll_events: Vec<FIXEvent>,
}

impl FixClient {
    /// Initializes the client and returns it alongside a handler for message passing.
    pub fn new(
        server_addr: SocketAddr,
        comp_id: String,
        target_comp_id: String,
        heart_bt_int: u16,
        encrypt_method: EncryptMethod,
    ) -> io::Result<(Self, FixClientHandler)> {
        let poll = Poll::new()?;
        let waker = Arc::new(Waker::new(poll.registry(), WAKE)?);

        let (lob_tx, lob_rx) = ringbuf::HeapRb::<FIXEvent>::new(256).split();
        let (outbound_tx, outbound_rx) = ringbuf::HeapRb::<FIXEvent>::new(1024).split();

        let comp_id_arc: Arc<str> = comp_id.into();

        let handler = FixClientHandler {
            comp_id: Arc::clone(&comp_id_arc),
            outbound_tx: Mutex::new(outbound_tx),
            lob_rx: Mutex::new(lob_rx),
            waker,
        };

        let client = Self {
            session: None,
            server_addr,
            comp_id: Arc::clone(&comp_id_arc),
            target_comp_id: Arc::from(target_comp_id),
            heart_bt_int,
            encrypt_method,
            outbound_rx,
            lob_tx,
            poll,
            poll_events: Vec::new(),
        };

        Ok((client, handler))
    }

    /// Establishes the TCP connection and sends the initial FIX Logon message.
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
            comp_id: Arc::clone(&self.comp_id),
            target_comp_id: Arc::clone(&self.target_comp_id),
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

    /// Runs the main blocking event loop. Polls for network I/O and checks session health continuously.
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

    /// Maintains session health by sending `Heartbeats` or `TestRequests` when idle, and closing the connection if unresponsive.
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

    /// Routes polled events to process either outbound application messages or inbound network traffic.
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

    /// Drains the outbound queue from the handler and pushes messages into the session's network write buffer.
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

    /// Flushes the write buffer to the TCP socket, updating poll interests to stop writing when empty.
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

    /// Reads from the TCP socket, processing session-level messages internally and forwarding business messages to the handler.
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
                    self.finalize_logon(Arc::clone(&event.comp_id), logon)
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

    /// Deregisters the stream and drops the session, closing the connection.
    fn close_session(&mut self) {
        if let Some(mut session) = self.session.take() {
            self.poll.registry().deregister(&mut session.stream).ok();
        }
    }

    /// Handles a ResendRequest by resending messages from the requested sequence range.
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

    /// Updates session parameters from the incoming Logon, or drops the connection if already logged in.
    fn finalize_logon(&mut self, _comp_id: Arc<str>, logon: Logon) {
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

    /// Directly queues a message for transmission and ensures the socket is polled for writing.
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

/// A handle for the application thread to send orders and receive FIX reports.
pub struct FixClientHandler {
    comp_id: Arc<str>,
    outbound_tx: Mutex<HeapProd<FIXEvent>>,
    lob_rx: Mutex<HeapCons<FIXEvent>>,
    waker: Arc<Waker>,
}

impl FixClientHandler {
    /// Converts an Order into a FIX `BusinessMessage`, queues it for sending, and wakes the client's event loop.
    pub fn send_message(&mut self, order: &Order) -> Result<(), &'static str> {
        let msg = BusinessMessage::from_order(order)?;

        let event = FIXEvent {
            comp_id: Arc::clone(&self.comp_id),
            payload: FIXPayload::Business(msg),
        };

        self.outbound_tx.lock().unwrap().try_push(event).ok();
        self.waker.wake().ok();

        Ok(())
    }

    /// Polls the inbound queue for the latest FIX reports from LOB.
    pub fn next_report(&mut self) -> Option<FIXEvent> {
        self.lob_rx.lock().unwrap().try_pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::lob_core::market_orders::{OrderSide, OrderType};
    use std::thread;

    #[test]
    #[ignore]
    fn fix_client_test() {
        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let (mut client, mut handler) = FixClient::new(
            addr,
            "CLIENT01".to_string(),
            "ENGINE01".to_string(),
            10,
            EncryptMethod::None,
        )
        .unwrap();

        client.connect().unwrap();

        let client_thread = thread::spawn(move || {
            client.run();
        });

        std::thread::sleep(Duration::from_secs(2));

        let order = Order {
            client_id: 0,
            order_id: 1,
            side: OrderSide::Bid,
            timestamp: 5,
            kind: OrderType::Limit {
                qty: 10,
                price: 666,
            },
        };

        let _ = handler.send_message(&order);

        client_thread.join().unwrap();
    }
}
