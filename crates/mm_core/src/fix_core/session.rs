use bytes::{Buf, BytesMut};
use mio::{Token, net::TcpStream};
use ringbuf::{HeapProd, traits::Producer};
use std::collections::{BTreeMap, VecDeque};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Instant;

use crate::fix_core::{
    helpers::{extract_message, write_fix_message},
    iterator::FixIterator,
    messages::{
        BusinessMessage, EngineMessage, FIX_MESSAGE_TYPE_EXECUTION_REPORT,
        FIX_MESSAGE_TYPE_HEARTBEAT, FIX_MESSAGE_TYPE_LOGON, FIX_MESSAGE_TYPE_NEW_ORDER,
        FIX_MESSAGE_TYPE_ORDER_CANCEL, FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT,
        FIX_MESSAGE_TYPE_RESEND_REQUEST, FIX_MESSAGE_TYPE_TEST_REQUEST, FIXEvent, FIXMessage,
        FIXPayload, ReportMessage, TAG_MSG_SEQ_NUM, TAG_MSG_TYPE, TAG_POSS_DUP_FLAG,
        TAG_SENDER_COMP_ID, execution_report::ExecutionReport, heartbeat::Heartbeat, logon::Logon,
        new_order_single::NewOrderSingle, order_cancel::OrderCancel,
        order_cancel_reject::OrderCancelReject, resend_request::ResendRequest,
        test_request::TestRequest, types::EncryptMethod,
    },
};

const MAX_BUFFER_SIZE: usize = 1024;
const MAX_TMP_BUFFER_SIZE: usize = 512;

pub struct Session {
    pub token: Token,
    pub stream: TcpStream,
    pub read_buffer: Vec<u8>,
    pub write_buffer: BytesMut,
    pub tmp: [u8; MAX_TMP_BUFFER_SIZE],
    pub tmp_end: usize,
    pub state: Option<SessionState>,
    pub last_sent: Instant,
    pub last_received: Instant,
    pub test_req_counter: u32,
    pub pending_test_req: Option<u32>,
}

#[derive(Clone)]
pub struct SessionState {
    pub comp_id: Arc<str>,
    pub target_comp_id: Arc<str>,
    pub inbound_seq_num: u32,
    pub outbound_seq_num: u32,
    pub logged_in: bool,
    pub encrypt_method: EncryptMethod,
    pub heart_bt_int: u16,
    pub sent_messages: BTreeMap<u32, FIXPayload>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            comp_id: Arc::from(""),
            target_comp_id: Arc::from(""),
            inbound_seq_num: 0,
            outbound_seq_num: 0,
            encrypt_method: EncryptMethod::None,
            heart_bt_int: 30,
            logged_in: false,
            sent_messages: BTreeMap::new(),
        }
    }
}

impl Session {
    pub fn new(token: Token, stream: TcpStream) -> Self {
        Self {
            token,
            stream,
            read_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
            write_buffer: BytesMut::with_capacity(MAX_BUFFER_SIZE),
            tmp: [0u8; MAX_TMP_BUFFER_SIZE],
            tmp_end: 0,
            state: None,
            last_sent: Instant::now(),
            last_received: Instant::now(),
            test_req_counter: 0,
            pending_test_req: None,
        }
    }

    pub fn poll(
        &mut self,
        events: &mut Vec<FIXEvent>,
        lob_tx: &mut HeapProd<FIXEvent>,
    ) -> Result<(), &'static str> {
        loop {
            if self.tmp_end >= MAX_TMP_BUFFER_SIZE && !self.read(events, lob_tx) {
                break;
            }

            match self.stream.read(&mut self.tmp[self.tmp_end..]) {
                Ok(0) => {
                    return Err("Peer closed connection");
                }
                Ok(n) => {
                    self.tmp_end += n;
                    self.last_received = Instant::now();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    self.read(events, lob_tx);
                    break;
                }
                Err(_e) => {
                    return Err("Error reading from stream");
                }
            }
        }

        Ok(())
    }

    fn read(&mut self, events: &mut Vec<FIXEvent>, lob_tx: &mut HeapProd<FIXEvent>) -> bool {
        if self.read_buffer.len() + self.tmp_end > MAX_BUFFER_SIZE {
            return false;
        }

        self.read_buffer
            .extend_from_slice(&self.tmp[..self.tmp_end]);
        self.tmp_end = 0;
        self.process_inbound_messages(events, lob_tx);

        true
    }

    fn process_inbound_messages(
        &mut self,
        engine_events: &mut Vec<FIXEvent>,
        lob_tx: &mut ringbuf::HeapProd<FIXEvent>,
    ) {
        while let Some(msg) = extract_message(&mut self.read_buffer) {
            let mut comp_id = None;
            let mut msg_seq_num = None;
            let mut msg_type: Option<u8> = None;
            let mut poss_dup_flag = false;

            for (tag, value) in FixIterator::new(&msg) {
                match tag {
                    TAG_SENDER_COMP_ID => {
                        comp_id = from_utf8(value).ok();
                    }
                    TAG_MSG_SEQ_NUM => {
                        msg_seq_num = from_utf8(value).ok().and_then(|v| v.parse().ok())
                    }
                    TAG_MSG_TYPE => msg_type = Some(value[0]),
                    TAG_POSS_DUP_FLAG => poss_dup_flag = value == b"Y",
                    _ => {}
                }
            }

            let Some(comp_id) = comp_id else { continue };
            let Some(msg_seq_num) = msg_seq_num else {
                continue;
            };

            if let Some(state) = self.state.as_mut() {
                let expected = state.inbound_seq_num + 1;
                if msg_seq_num == expected {
                    state.inbound_seq_num = msg_seq_num;
                } else if msg_seq_num > expected {
                    self.send_message(
                        FIXPayload::Engine(EngineMessage::ResendRequest(ResendRequest {
                            begin_seq_no: expected,
                            end_seq_no: 0,
                        })),
                        None,
                        false,
                    )
                    .ok();

                    if msg_type != Some(FIX_MESSAGE_TYPE_RESEND_REQUEST)
                        && msg_type != Some(FIX_MESSAGE_TYPE_LOGON)
                    {
                        continue;
                    }
                } else if !poss_dup_flag {
                    continue;
                }
            }

            let parsed = match msg_type {
                Some(FIX_MESSAGE_TYPE_LOGON) => {
                    Logon::from_bytes(&msg).map(|m| FIXPayload::Engine(EngineMessage::Logon(m)))
                }
                Some(FIX_MESSAGE_TYPE_HEARTBEAT) => Heartbeat::from_bytes(&msg)
                    .map(|m| FIXPayload::Engine(EngineMessage::Heartbeat(m))),
                Some(FIX_MESSAGE_TYPE_TEST_REQUEST) => TestRequest::from_bytes(&msg)
                    .map(|m| FIXPayload::Engine(EngineMessage::TestRequest(m))),
                Some(FIX_MESSAGE_TYPE_RESEND_REQUEST) => ResendRequest::from_bytes(&msg)
                    .map(|m| FIXPayload::Engine(EngineMessage::ResendRequest(m))),
                Some(FIX_MESSAGE_TYPE_NEW_ORDER) => NewOrderSingle::from_bytes(&msg)
                    .map(|m| FIXPayload::Business(BusinessMessage::NewOrderSingle(m))),
                Some(FIX_MESSAGE_TYPE_ORDER_CANCEL) => OrderCancel::from_bytes(&msg)
                    .map(|m| FIXPayload::Business(BusinessMessage::OrderCancel(m))),
                Some(FIX_MESSAGE_TYPE_EXECUTION_REPORT) => ExecutionReport::from_bytes(&msg)
                    .map(|m| FIXPayload::Report(ReportMessage::ExecutionReport(m))),
                Some(FIX_MESSAGE_TYPE_ORDER_CANCEL_REJECT) => OrderCancelReject::from_bytes(&msg)
                    .map(|m| FIXPayload::Report(ReportMessage::OrderCancelReject(m))),
                _ => Err("Unsupported MsgType"),
            };

            println!("Process Inbound Message | {:?}", parsed);

            if let Ok(payload) = parsed {
                let comp_id: Arc<str> = Arc::from(comp_id);
                let req = FIXEvent { comp_id, payload };
                match req.payload {
                    FIXPayload::Engine(_) => engine_events.push(req),
                    FIXPayload::Business(_) | FIXPayload::Report(_) => {
                        lob_tx.try_push(req).ok();
                    }
                }
            }
        }
    }

    pub fn send_message(
        &mut self,
        payload: FIXPayload,
        override_seq_num: Option<u32>,
        poss_dup_flag: bool,
    ) -> Result<(), &'static str> {
        let state = self.state.as_mut().ok_or("Missing SenderCompID")?;
        let sender_comp_id = state.target_comp_id.clone();
        let target_comp_id = Arc::clone(&state.comp_id);

        let seq_num = match override_seq_num {
            Some(seq) => seq,
            None => {
                state.outbound_seq_num += 1;
                state.outbound_seq_num
            }
        };

        let mut body = payload.as_bytes();

        println!("Send Message | {:?}", payload);

        if poss_dup_flag {
            let mut new_body = Vec::new();
            new_body.extend_from_slice(b"43=Y\x01");
            new_body.extend_from_slice(&body);
            body = new_body;
        }

        let msg = write_fix_message(
            payload.message_type(),
            &seq_num,
            &sender_comp_id,
            &target_comp_id,
            &body,
        );

        if override_seq_num.is_none() {
            state.sent_messages.insert(seq_num, payload);
        }

        self.write_buffer.extend_from_slice(&msg);
        self.last_sent = Instant::now();

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), &'static str> {
        loop {
            if self.write_buffer.is_empty() {
                break;
            }

            match self.stream.write(&self.write_buffer) {
                Ok(0) => return Err("Connection closed"),
                Ok(n) => {
                    self.last_sent = Instant::now();
                    self.write_buffer.advance(n);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => return Err("Write error"),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mio::net::TcpListener;
    use std::net::SocketAddr;

    fn make_session() -> Session {
        let listener_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(listener_addr).expect("err");
        let addr = listener.local_addr().expect("err");

        let _client = TcpStream::connect(addr).expect("err");
        let (server, _) = listener.accept().expect("err");

        Session::new(Token(0), server)
    }

    #[test]
    fn test_extract_message_empty_body() {
        let mut session = make_session();

        let msg_type = b'0';
        let seq_num = 1;
        let sender = "A";
        let target = "B";
        let empty_body: Vec<u8> = Vec::new();

        let msg = write_fix_message(msg_type, &seq_num, sender, target, &empty_body);

        session.read_buffer.extend_from_slice(&msg);

        let out = extract_message(&mut session.read_buffer).expect("err");
        assert_eq!(out, msg);
        assert!(session.read_buffer.is_empty());
    }

    #[test]
    fn test_extract_message_checksum_mismatch() {
        let mut session = make_session();

        let msg = b"8=FIX.4.2\x019=5\x0135=D\x0110=000\x01";
        session.read_buffer.extend_from_slice(msg);

        assert!(extract_message(&mut session.read_buffer).is_none());
    }
}
