use netlib::fix_core::helpers::{print_message, write_header, write_trailer};
use std::io::{Result, Write};
use std::net::TcpStream;
use zerocopy::IntoBytes;

pub struct Session {
    pub inbound_sequence_number: u32,
    pub logged_in: bool,
    pub outbound_sequence_number: u32,
    pub sender_comp_id: String,
    pub stream: TcpStream,
    pub target_comp_id: String,
    pub write_buf: Vec<u8>,
}

impl Session {
    pub fn connect() -> Result<Self> {
        let stream = TcpStream::connect("127.0.0.1:34254")?;

        Ok(Session {
            inbound_sequence_number: 1,
            logged_in: false,
            outbound_sequence_number: 1,
            sender_comp_id: "CLIENT01".to_string(),
            stream,
            target_comp_id: "ENGINE01".to_string(),
            write_buf: Vec::new(),
        })
    }

    pub fn send_message(&mut self, msg_type: &[u8], body: Vec<u8>) {
        let body_length = self.encode_fix_body(msg_type, body);
        self.encode_fix_wrapper(body_length);
        self.send_fix_message();
    }

    fn encode_fix_body(&mut self, msg_type: &[u8], body: Vec<u8>) -> usize {
        self.write_buf.clear();

        write_header(
            &mut self.write_buf,
            msg_type,
            &self.outbound_sequence_number,
            &self.sender_comp_id,
            &self.target_comp_id,
        );

        self.write_buf.extend_from_slice(&body);
        self.write_buf.len()
    }

    fn encode_fix_wrapper(&mut self, body_length: usize) {
        let mut itoa_buf = itoa::Buffer::new();
        let mut final_buf = Vec::with_capacity(body_length + 64);

        final_buf.extend_from_slice(b"8=FIX.4.2\x01");

        final_buf.extend_from_slice(b"9=");
        final_buf.extend_from_slice(itoa_buf.format(body_length).as_bytes());
        final_buf.push(0x01);

        final_buf.extend_from_slice(&self.write_buf);
        self.write_buf = final_buf;

        write_trailer(&mut self.write_buf);
        print_message(&self.write_buf);
    }

    fn send_fix_message(&mut self) {
        self.stream.write_all(&self.write_buf).expect("err");
        self.outbound_sequence_number = self.outbound_sequence_number.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, str::from_utf8};

    #[test]
    #[ignore]
    fn test_header() {
        let mut session = Session::connect().expect("err");

        let body = Vec::new();
        session.send_message("D".as_bytes(), body);
        print_message(&session.write_buf);
    }

    #[test]
    fn test_fix_fields() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("err");
        let addr = listener.local_addr().expect("err");

        let client = TcpStream::connect(addr).expect("err");
        let (server, _) = listener.accept().expect("err");

        let mut session = Session {
            inbound_sequence_number: 1,
            logged_in: false,
            outbound_sequence_number: 1,
            sender_comp_id: "CLIENT01".to_string(),
            target_comp_id: "ENGINE01".to_string(),
            stream: server,
            write_buf: Vec::new(),
        };

        let body = Vec::new();

        let body_len = session.encode_fix_body(b"D", body);
        session.encode_fix_wrapper(body_len);

        let s = from_utf8(&session.write_buf).expect("err");

        assert!(s.contains("8=FIX.4.2"));
        assert!(s.contains("35=D"));
        assert!(s.contains("34=1"));
        assert!(s.contains("49=CLIENT01"));
        assert!(s.contains("56=ENGINE01"));
        assert!(s.contains("10="));

        drop(client);
    }
}
