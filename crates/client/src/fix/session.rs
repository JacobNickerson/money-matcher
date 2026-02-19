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
    pub itoa_buf: itoa::Buffer,
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
            itoa_buf: itoa::Buffer::new(),
            write_buf: Vec::new(),
        })
    }

    pub fn send_message(&mut self, msg_type: &[u8], body: Vec<u8>) {
        self.write_buf.clear();

        self.write_header(msg_type);
        self.write_buf.extend_from_slice(&body);
        let body_length = self.write_buf.len();

        let mut final_buf = Vec::with_capacity(body_length + 64);

        // Begin String
        final_buf.extend_from_slice(b"8=FIX.4.2\x01");

        // Body Length
        final_buf.extend_from_slice(b"9=");
        final_buf.extend_from_slice(self.itoa_buf.format(body_length).as_bytes());
        final_buf.push(0x01);

        final_buf.extend_from_slice(&self.write_buf);

        self.write_buf = final_buf;

        self.write_trailer();

        self.stream.write_all(&self.write_buf).unwrap();

        self.outbound_sequence_number += 1;
    }

    pub fn write_header(&mut self, msg_type: &[u8]) {
        // Message Type
        self.write_buf.extend_from_slice(b"35=");
        self.write_buf.extend_from_slice(msg_type);
        self.write_buf.push(0x01);

        // Message Sequence Number
        self.write_buf.extend_from_slice(b"34=");
        self.write_buf.extend_from_slice(
            self.itoa_buf
                .format(self.outbound_sequence_number)
                .as_bytes(),
        );
        self.write_buf.push(0x01);

        // Sender Comp ID
        self.write_buf.extend_from_slice(b"49=");
        self.write_buf
            .extend_from_slice(self.sender_comp_id.as_bytes());
        self.write_buf.push(0x01);

        // Sending Time
        self.write_buf.extend_from_slice(b"52=");
        self.write_buf.extend_from_slice("SENDINGTIME".as_bytes());
        self.write_buf.push(0x01);

        // Target Comp ID
        self.write_buf.extend_from_slice(b"56=");
        self.write_buf
            .extend_from_slice(self.target_comp_id.as_bytes());
        self.write_buf.push(0x01);
    }

    pub fn calculate_checksum(&mut self) -> u32 {
        let mut sum: u32 = 0;

        for byte in &self.write_buf {
            sum += *byte as u32;
        }

        sum % 256
    }

    pub fn write_trailer(&mut self) {
        // Checksum
        let checksum: u32 = self.calculate_checksum();
        self.write_buf.extend_from_slice(b"10=");
        self.write_buf.push(b'0' + (checksum / 100) as u8);
        self.write_buf.push(b'0' + ((checksum / 10) % 10) as u8);
        self.write_buf.push(b'0' + (checksum % 10) as u8);
        self.write_buf.push(0x01);
    }

    pub fn print_message(&self) {
        let mut output = String::new();

        for &byte in &self.write_buf {
            let c = if byte == 0x01 { b'|' } else { byte };
            output.push(c as char);
        }

        println!("{}", output);
    }

    pub fn write_new_order(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        // ClOrdID (Tag 11) - Maximum 20 characters
        buf.extend_from_slice(b"11=");
        buf.extend_from_slice(self.itoa_buf.format(1).as_bytes());
        buf.push(0x01);

        // HandlInst (Tag 21) - Required by protocol, ignored by ISE
        buf.extend_from_slice(b"21=1\x01");

        // OrderQty (Tag 38)
        buf.extend_from_slice(b"38=");
        buf.extend_from_slice(self.itoa_buf.format(10).as_bytes());
        buf.push(0x01);

        // OrdType
        buf.extend_from_slice(b"40=2\x01");

        // Price
        buf.extend_from_slice(b"44=");
        buf.extend_from_slice(self.itoa_buf.format(10).as_bytes());
        buf.push(0x01);

        // Side
        buf.extend_from_slice(b"54=");
        buf.extend_from_slice(self.itoa_buf.format(0).as_bytes());
        buf.push(0x01);

        // Symbol
        buf.extend_from_slice(b"55=XYZ\x01");

        // TransactTime
        buf.extend_from_slice(b"60=TIME\x01");

        // OpenClose
        buf.extend_from_slice(b"77=O\x01");

        // SecurityType
        buf.extend_from_slice(b"167=OPT\x01");

        // MaturityMonthYear
        buf.extend_from_slice(b"200=202601\x01");

        // PutOrCall
        buf.extend_from_slice(b"201=");
        buf.extend_from_slice(self.itoa_buf.format(1).as_bytes());
        buf.push(0x01);

        // StrikePrice
        buf.extend_from_slice(b"202=");
        buf.extend_from_slice(self.itoa_buf.format(10).as_bytes());
        buf.push(0x01);

        // CustomerOrFirm
        buf.extend_from_slice(b"204=");
        buf.extend_from_slice(self.itoa_buf.format(0).as_bytes());
        buf.push(0x01);

        // MaturityDay
        buf.extend_from_slice(b"205=");
        buf.extend_from_slice(self.itoa_buf.format(10).as_bytes());
        buf.push(0x01);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header() {
        let mut session = Session::connect().unwrap();

        let body = session.write_new_order();
        session.send_message("D".as_bytes(), body);
        session.print_message();
    }
}
