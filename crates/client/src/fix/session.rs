use mio::{Registry, event::Event, net::TcpStream};
use netlib::fix_core::{
    helpers::{extract_message, print_message, write_fix_message},
    iterator::FixIterator,
    messages::FixFrame,
};
use ringbuf::{HeapCons, traits::Consumer};
use std::{
    io::{self, Read, Result, Write},
    net::SocketAddr,
};

use mio::{Events, Interest, Poll, Token, Waker};
use zerocopy::IntoBytes;
const SERVER_CONN: Token = Token(0);
const WAKE: Token = Token(1);

pub struct Session {
    pub inbound_sequence_number: u32,
    pub logged_in: bool,
    pub outbound_sequence_number: u32,
    pub sender_comp_id: String,
    pub stream: TcpStream,
    pub target_comp_id: String,
    pub write_buffer: Vec<u8>,
    pub read_buffer: Vec<u8>,
    pub session_rx: HeapCons<FixFrame>,
    pub poll: Poll,
    tmp: [u8; 4096],
    tmp_end: usize,
}

impl Session {
    pub fn connect(addr: SocketAddr, poll: Poll, session_rx: HeapCons<FixFrame>) -> Result<Self> {
        let mut stream = TcpStream::connect(addr)?;

        {
            let registry = poll.registry();
            registry.register(&mut stream, SERVER_CONN, Interest::READABLE)?;
        }

        Ok(Self {
            inbound_sequence_number: 1,
            logged_in: false,
            outbound_sequence_number: 1,
            sender_comp_id: "CLIENT01".to_string(),
            stream,
            target_comp_id: "ENGINE01".to_string(),
            write_buffer: Vec::new(),
            read_buffer: Vec::new(),
            session_rx,
            poll,
            tmp: [0u8; 4096],
            tmp_end: 0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut events = Events::with_capacity(1024);
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                self.handle_event(event);
            }
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event.token() {
            SERVER_CONN => self.handle_server_event(event),
            WAKE => self.process_requests(),
            _ => (),
        }
    }

    fn handle_server_event(&mut self, event: &Event) {
        if event.is_readable() {
            self.handle_server_readable();
        }

        if event.is_writable() {
            self.handle_server_writable();
        }
    }

    fn handle_server_readable(&mut self) {
        loop {
            match self.stream.read(&mut self.tmp[self.tmp_end..]) {
                Ok(0) => {
                    panic!("Connection closed by server");
                }
                Ok(n) => {
                    self.tmp_end += n;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    panic!("Read error: {}", e);
                }
            }
        }

        self.read_buffer
            .extend_from_slice(&self.tmp[..self.tmp_end]);
        self.tmp_end = 0;

        self.process_replies();
    }

    fn process_replies(&mut self) {
        while let Some(msg) = extract_message(&mut self.read_buffer) {
            print_message(&msg);
        }
    }

    fn handle_server_writable(&mut self) {
        match self.stream.write(&self.write_buffer) {
            Ok(n) => {
                self.write_buffer.drain(..n);

                if self.write_buffer.is_empty() {
                    self.poll
                        .registry()
                        .reregister(&mut self.stream, SERVER_CONN, Interest::READABLE)
                        .unwrap();
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("Write error: {}", e),
        }
    }

    fn process_requests(&mut self) {
        while let Some(cmd) = self.session_rx.try_pop() {
            write_fix_message(
                &mut self.write_buffer,
                cmd.msg_type,
                &self.outbound_sequence_number,
                &self.sender_comp_id,
                &self.target_comp_id,
                &cmd.body,
            );

            self.outbound_sequence_number = self.outbound_sequence_number.wrapping_add(1);

            self.poll
                .registry()
                .reregister(
                    &mut self.stream,
                    SERVER_CONN,
                    Interest::READABLE | Interest::WRITABLE,
                )
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netlib::fix_core::{helpers::print_message, messages::FIX_MESSAGE_TYPE_NEW_ORDER};
    use std::{net::TcpListener, str::from_utf8};

    //#[test]
    //#[ignore]
    //fn test_header() {
    //    let mut session = Session::connect().expect("err");
    //
    //    let body = Vec::new();
    //    session.send_message(FIX_MESSAGE_TYPE_NEW_ORDER, body);
    //    print_message(&session.write_buffer);
    //}
    //
    //#[test]
    //fn test_fix_fields() {
    //    let listener = TcpListener::bind("127.0.0.1:0").expect("err");
    //    let addr = listener.local_addr().expect("err");
    //
    //    let client = TcpStream::connect(addr).expect("err");
    //    let (server, _) = listener.accept().expect("err");
    //
    //    let mut session = Session {
    //        inbound_sequence_number: 1,
    //        logged_in: false,
    //        outbound_sequence_number: 1,
    //        sender_comp_id: "CLIENT01".to_string(),
    //        target_comp_id: "ENGINE01".to_string(),
    //        stream: server,
    //        write_buffer: Vec::new(),
    //    };
    //
    //    let body = Vec::new();
    //
    //    write_fix_message(
    //        &mut session.write_buffer,
    //        &FIX_MESSAGE_TYPE_NEW_ORDER,
    //        &session.outbound_sequence_number,
    //        &session.sender_comp_id,
    //        &session.target_comp_id,
    //        &body,
    //    );
    //
    //    let s = from_utf8(&session.write_buffer).expect("err");
    //
    //    assert!(s.contains("8=FIX.4.2"));
    //    assert!(s.contains("35=D"));
    //    assert!(s.contains("34=1"));
    //    assert!(s.contains("49=CLIENT01"));
    //    assert!(s.contains("56=ENGINE01"));
    //    assert!(s.contains("10="));
    //
    //    drop(client);
    //}
}
