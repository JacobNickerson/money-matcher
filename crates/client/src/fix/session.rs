use netlib::fix_core::helpers::{
    get_maturity_month_year, get_timestamp, print_message, write_header, write_trailer,
};
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
        let mut itoa_buf = itoa::Buffer::new();

        self.write_buf.clear();

        write_header(
            &mut self.write_buf,
            msg_type,
            &self.outbound_sequence_number,
            &self.sender_comp_id,
            &self.target_comp_id,
        );
        self.write_buf.extend_from_slice(&body);
        let body_length = self.write_buf.len();

        let mut final_buf = Vec::with_capacity(body_length + 64);

        // Begin String
        final_buf.extend_from_slice(b"8=FIX.4.2\x01");

        // Body Length
        final_buf.extend_from_slice(b"9=");
        final_buf.extend_from_slice(itoa_buf.format(body_length).as_bytes());
        final_buf.push(0x01);

        final_buf.extend_from_slice(&self.write_buf);

        self.write_buf = final_buf;

        write_trailer(&mut self.write_buf);

        print_message(&self.write_buf);

        self.stream.write_all(&self.write_buf).unwrap();

        self.outbound_sequence_number += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header() {
        let mut session = Session::connect().unwrap();

        let body = Vec::new();
        session.send_message("D".as_bytes(), body);
        print_message(&session.write_buf);
    }

    //    #[test]
    //    fn test_body() {
    //        let mut session = Session::connect().unwrap();
    //
    //        let body = write_new_order();
    //        session.send_message("D".as_bytes(), body);
    //        print_message(&session.write_buf);
    //    }
}
