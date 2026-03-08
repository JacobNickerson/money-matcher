use mio::{Token, net::TcpStream};
use netlib::fix_core::helpers::{extract_message, write_fix_message};
use netlib::fix_core::messages::execution_report::ExecutionReport;
use netlib::fix_core::messages::heartbeat::Heartbeat;
use netlib::fix_core::messages::logon::Logon;
use netlib::fix_core::messages::test_request::TestRequest;
use netlib::fix_core::messages::types::{
    CustomerOrFirm, EncryptMethod, ExecTransType, ExecType, OpenClose, OrdStatus, PutOrCall, Side,
};
use netlib::fix_core::messages::{
    FIX_MESSAGE_TYPE_EXECUTION_REPORT, FIX_MESSAGE_TYPE_HEARTBEAT, FIX_MESSAGE_TYPE_LOGON,
    FIX_MESSAGE_TYPE_TEST_REQUEST, TAG_CL_ORD_ID, TAG_CUM_QTY, TAG_CUSTOMER_OR_FIRM,
    TAG_ENCRYPT_METHOD, TAG_EXEC_ID, TAG_EXEC_TRANS_TYPE, TAG_EXEC_TYPE, TAG_HEART_BT_INT,
    TAG_LEAVES_QTY, TAG_MATURITY_DATE, TAG_MSG_TYPE, TAG_OPEN_CLOSE, TAG_ORD_STATUS, TAG_ORDER_ID,
    TAG_ORDER_QTY, TAG_PUT_OR_CALL, TAG_SECURITY_ID, TAG_SECURITY_TYPE, TAG_SENDER_COMP_ID,
    TAG_SIDE, TAG_STRIKE_PRICE, TAG_SYMBOL, TAG_TEST_REQ_ID,
};
use netlib::fix_core::messages::{FIXReply, FIXReplyMessage};
use netlib::fix_core::{iterator::FixIterator, messages::FixMessage};
use std::collections::VecDeque;
use std::time::Instant;
use std::{
    io::{Read, Write},
    str::from_utf8,
};

const MAX_BUFFER_SIZE: usize = 1024;
const MAX_TMP_BUFFER_SIZE: usize = 512;

pub struct Session {
    token: Token,
    pub(crate) stream: TcpStream,
    read_buffer: Vec<u8>,
    pub(crate) write_buffer: VecDeque<u8>,
    tmp: [u8; MAX_TMP_BUFFER_SIZE],
    tmp_end: usize,
    pub state: Option<SessionState>,
    pub last_sent: Instant,     // Client -> Server
    pub last_received: Instant, // Server -> Client
    pub(crate) test_req_counter: u32,
    pub pending_test_req: Option<u32>,
}

#[derive(Clone)]
pub struct SessionState {
    pub comp_id: String,
    pub target_comp_id: String,
    pub inbound_seq_num: u32,
    pub outbound_seq_num: u32,
    pub logged_in: bool,
    pub encrypt_method: EncryptMethod,
    pub heart_bt_int: u16,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            comp_id: String::new(),
            target_comp_id: String::new(),
            inbound_seq_num: 0,
            outbound_seq_num: 1,
            encrypt_method: EncryptMethod::None,
            heart_bt_int: 30,
            logged_in: false,
        }
    }
}

impl Session {
    pub fn new(token: Token, stream: TcpStream) -> Self {
        Self {
            token,
            stream,
            read_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
            write_buffer: VecDeque::with_capacity(MAX_BUFFER_SIZE),
            tmp: [0u8; MAX_TMP_BUFFER_SIZE],
            tmp_end: 0,
            state: None,
            last_sent: Instant::now(),
            last_received: Instant::now(),
            test_req_counter: 10000,
            pending_test_req: None,
        }
    }

    pub fn poll(&mut self, events: &mut Vec<FIXReply>) -> Result<(), &'static str> {
        loop {
            if self.tmp_end >= MAX_TMP_BUFFER_SIZE && !self.read(events) {
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
                    self.read(events);
                    break;
                }
                Err(_e) => {
                    return Err("Error reading from stream");
                }
            }
        }

        Ok(())
    }

    fn read(&mut self, events: &mut Vec<FIXReply>) -> bool {
        if self.read_buffer.len() + self.tmp_end > MAX_BUFFER_SIZE {
            return false;
        }

        self.read_buffer
            .extend_from_slice(&self.tmp[..self.tmp_end]);
        self.tmp_end = 0;
        self.process_messages(events);

        true
    }

    fn process_messages(&mut self, events: &mut Vec<FIXReply>) {
        while let Some(msg) = extract_message(&mut self.read_buffer) {
            let mut msg_type = None;

            for (tag, value) in FixIterator::new(&msg) {
                if tag == TAG_MSG_TYPE {
                    msg_type = Some(value);
                    break;
                }
            }

            let event: Option<FIXReply> = match msg_type {
                Some(FIX_MESSAGE_TYPE_LOGON) => self.handle_logon(&msg).ok(),
                Some(FIX_MESSAGE_TYPE_EXECUTION_REPORT) => self.handle_execution_report(&msg).ok(),
                Some(FIX_MESSAGE_TYPE_TEST_REQUEST) => self.handle_test_request(&msg).ok(),
                Some(FIX_MESSAGE_TYPE_HEARTBEAT) => self.handle_heartbeat(&msg).ok(),

                _ => None,
            };

            events.extend(event);
        }
    }

    pub fn handle_request<T>(&mut self, request: T) -> Result<(), &'static str>
    where
        T: FixMessage,
    {
        let state = self.state.as_ref().ok_or("Can't find comp_id")?;
        let sender_comp_id = state.comp_id.clone();
        let target_comp_id = state.target_comp_id.clone();

        let msg = write_fix_message(
            T::MESSAGE_TYPE,
            &1_u32,
            &sender_comp_id,
            &target_comp_id,
            &request.as_bytes(),
        );

        self.write_buffer.extend(msg);
        self.last_sent = Instant::now();

        Ok(())
    }

    pub fn send_requests(&mut self) -> Result<(), &'static str> {
        loop {
            if self.write_buffer.is_empty() {
                break;
            }

            let slice = self.write_buffer.make_contiguous();
            match self.stream.write(slice) {
                Ok(n) => {
                    println!("wrote {} bytes", n);
                    self.write_buffer.drain(..n);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    println!("write error: {}", e);
                    return Err("Write error");
                }
            }
        }

        Ok(())
    }

    fn handle_logon(&mut self, msg: &Vec<u8>) -> Result<FIXReply, &'static str> {
        let mut comp_id: Option<String> = None;
        let mut encrypt_method: Option<EncryptMethod> = None;
        let mut heart_bt_int: Option<u16> = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_SENDER_COMP_ID => {
                    comp_id = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_ENCRYPT_METHOD => {
                    encrypt_method = value
                        .first()
                        .copied()
                        .and_then(|b| EncryptMethod::try_from(b).ok());
                }
                TAG_HEART_BT_INT => {
                    heart_bt_int = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                _ => {}
            }
        }

        let comp_id = comp_id.ok_or("Missing SenderCompID")?;

        Ok(FIXReply {
            comp_id: comp_id.clone(),
            message: FIXReplyMessage::Logon(Logon {
                encrypt_method: encrypt_method.unwrap_or_default(),
                heart_bt_int: heart_bt_int.unwrap_or(30),
            }),
        })
    }

    fn handle_execution_report(&mut self, msg: &Vec<u8>) -> Result<FIXReply, &'static str> {
        let comp_id = self
            .state
            .as_ref()
            .ok_or("Can't find comp_id")?
            .comp_id
            .clone();

        let mut cl_ord_id = None;
        let mut cum_qty = None;
        let mut exec_id = None;
        let mut exec_trans_type = None;
        let mut order_id = None;
        let mut order_qty = None;
        let mut ord_status = None;
        let mut security_id = None;
        let mut side = None;
        let mut symbol = None;
        let mut open_close = None;
        let mut exec_type = None;
        let mut leaves_qty = None;
        let mut security_type = None;
        let mut put_or_call = None;
        let mut strike_price = None;
        let mut customer_or_firm = None;
        let mut maturity_date = None;

        for (tag, value) in FixIterator::new(msg) {
            match tag {
                TAG_CL_ORD_ID => {
                    cl_ord_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_CUM_QTY => {
                    cum_qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_EXEC_ID => {
                    exec_id = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_EXEC_TRANS_TYPE => {
                    exec_trans_type = value
                        .first()
                        .copied()
                        .and_then(|b| ExecTransType::try_from(b).ok());
                }
                TAG_ORDER_ID => {
                    order_id = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_ORDER_QTY => {
                    order_qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_ORD_STATUS => {
                    ord_status = value
                        .first()
                        .copied()
                        .and_then(|b| OrdStatus::try_from(b).ok());
                }
                TAG_SECURITY_ID => {
                    security_id = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_SIDE => {
                    side = value.first().copied().and_then(|b| Side::try_from(b).ok());
                }
                TAG_SYMBOL => {
                    symbol = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_OPEN_CLOSE => {
                    open_close = value
                        .first()
                        .copied()
                        .and_then(|b| OpenClose::try_from(b).ok());
                }
                TAG_EXEC_TYPE => {
                    exec_type = value
                        .first()
                        .copied()
                        .and_then(|b| ExecType::try_from(b).ok());
                }
                TAG_LEAVES_QTY => {
                    leaves_qty = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_SECURITY_TYPE => {
                    security_type = from_utf8(value).ok().map(str::to_owned);
                }
                TAG_PUT_OR_CALL => {
                    put_or_call = value
                        .first()
                        .copied()
                        .and_then(|b| PutOrCall::try_from(b).ok());
                }
                TAG_STRIKE_PRICE => {
                    strike_price = from_utf8(value).ok().and_then(|v| v.parse().ok());
                }
                TAG_CUSTOMER_OR_FIRM => {
                    customer_or_firm = value
                        .first()
                        .copied()
                        .and_then(|b| CustomerOrFirm::try_from(b).ok());
                }
                TAG_MATURITY_DATE => {
                    maturity_date = from_utf8(value).ok().map(str::to_owned);
                }
                _ => {}
            }
        }

        let execution_report = ExecutionReport {
            cl_ord_id: cl_ord_id.ok_or("Missing 11")?,
            cum_qty: cum_qty.ok_or("Missing 14")?,
            exec_id: exec_id.ok_or("Missing 17")?,
            exec_trans_type: exec_trans_type.ok_or("Missing 20")?,
            order_id: order_id.ok_or("Missing 37")?,
            order_qty: order_qty.ok_or("Missing 38")?,
            ord_status: ord_status.ok_or("Missing 39")?,
            security_id: security_id.ok_or("Missing 48")?,
            side: side.ok_or("Missing 54")?,
            symbol: symbol.ok_or("Missing 55")?,
            open_close: open_close.ok_or("Missing 77")?,
            exec_type: exec_type.ok_or("Missing 150")?,
            leaves_qty: leaves_qty.ok_or("Missing 151")?,
            security_type: security_type.ok_or("Missing 167")?,
            put_or_call: put_or_call.ok_or("Missing 201")?,
            strike_price: strike_price.ok_or("Missing 202")?,
            customer_or_firm: customer_or_firm.ok_or("Missing 204")?,
            maturity_date: maturity_date.ok_or("Missing 541")?,
        };

        println!("Read Execution Report | {:?}", execution_report);

        Ok(FIXReply {
            comp_id,
            message: FIXReplyMessage::ExecutionReport(execution_report),
        })
    }

    fn handle_heartbeat(&mut self, msg: &Vec<u8>) -> Result<FIXReply, &'static str> {
        let comp_id = self
            .state
            .as_ref()
            .ok_or("Can't find comp_id")?
            .comp_id
            .clone();
        let mut test_req_id: Option<u32> = None;
        for (tag, value) in FixIterator::new(msg) {
            if tag == TAG_TEST_REQ_ID {
                test_req_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
            }
        }

        let heartbeat = Heartbeat { test_req_id };
        println!("Read heartbeat | {:?}", heartbeat);

        Ok(FIXReply {
            comp_id,
            message: FIXReplyMessage::Heartbeat(heartbeat),
        })
    }

    fn handle_test_request(&mut self, msg: &Vec<u8>) -> Result<FIXReply, &'static str> {
        let comp_id = self
            .state
            .as_ref()
            .ok_or("Can't find comp_id")?
            .comp_id
            .clone();
        let mut test_req_id: Option<u32> = None;

        for (tag, value) in FixIterator::new(msg) {
            if tag == TAG_TEST_REQ_ID {
                test_req_id = from_utf8(value).ok().and_then(|v| v.parse().ok());
            }
        }

        let test_request = TestRequest {
            test_req_id: test_req_id.unwrap_or(0),
        };

        Ok(FIXReply {
            comp_id,
            message: FIXReplyMessage::TestRequest(test_request),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mio::net::TcpListener;
    use netlib::fix_core::helpers::write_trailer;
    use std::net::SocketAddr;

    fn make_session() -> Session {
        let listener_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(listener_addr).expect("err");
        let addr = listener.local_addr().expect("err");

        let client = TcpStream::connect(addr).expect("err");
        Session::new(Token(0), client)
    }

    #[test]
    fn test_extract_message_valid_checksum() {
        let mut session = make_session();

        let body = b"35=A\x01";
        let body_len = body.len();

        let mut msg = Vec::new();
        msg.extend_from_slice(b"8=FIX.4.2\x01");
        msg.extend_from_slice(b"9=");
        msg.extend_from_slice(body_len.to_string().as_bytes());
        msg.push(0x01);
        msg.extend_from_slice(body);

        write_trailer(&mut msg);

        session.read_buffer.extend_from_slice(&msg);

        let out = extract_message(&mut session.read_buffer).expect("err");
        assert_eq!(out, msg);
    }

    #[test]
    fn test_extract_message_checksum_mismatch() {
        let mut session = make_session();

        let msg = b"8=FIX.4.2\x019=5\x0135=A\x0110=000\x01";
        session.read_buffer.extend_from_slice(msg);

        assert!(extract_message(&mut session.read_buffer).is_none());
    }
}
