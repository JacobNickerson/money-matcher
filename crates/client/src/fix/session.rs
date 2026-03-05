use mio::{event::Event, net::TcpStream};
use netlib::fix_core::{
    helpers::{extract_message, print_message, write_fix_message},
    messages::FixFrame,
};
use ringbuf::{HeapCons, traits::Consumer};
use std::{
    io::{Read, Result, Write},
    net::SocketAddr,
};

use mio::{Events, Interest, Poll, Token};
const SERVER_CONN: Token = Token(0);
const WAKE: Token = Token(1);
const MAX_BUFFER_SIZE: usize = 1024;
const MAX_TMP_BUFFER_SIZE: usize = 512;

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
    tmp: [u8; MAX_TMP_BUFFER_SIZE],
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
            write_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
            read_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
            session_rx,
            poll,
            tmp: [0u8; MAX_TMP_BUFFER_SIZE],
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
            if self.tmp_end >= MAX_TMP_BUFFER_SIZE {
                if !self.read() {
                    break;
                }
            }

            match self.stream.read(&mut self.tmp[self.tmp_end..]) {
                Ok(0) => {
                    panic!("Connection closed by server");
                }
                Ok(n) => {
                    self.tmp_end += n;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    self.read();
                    break;
                }
                Err(e) => break,
            }
        }
    }

    fn read(&mut self) -> bool {
        if self.read_buffer.len() + self.tmp_end > MAX_BUFFER_SIZE {
            return false;
        }

        self.read_buffer
            .extend_from_slice(&self.tmp[..self.tmp_end]);
        self.tmp_end = 0;
        self.process_replies();

        true
    }

    fn process_replies(&mut self) {
        while let Some(msg) = extract_message(&mut self.read_buffer) {
            self.inbound_sequence_number = self.inbound_sequence_number.wrapping_add(1);
            print_message(&msg);
        }
    }

    fn handle_server_writable(&mut self) {
        loop {
            if self.write_buffer.is_empty() {
                self.poll
                    .registry()
                    .reregister(&mut self.stream, SERVER_CONN, Interest::READABLE)
                    .unwrap();
                self.process_requests();
                break;
            }

            match self.stream.write(&self.write_buffer) {
                Ok(n) => {
                    self.write_buffer.drain(..n);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("Write error: {}", e),
            }
        }
    }

    fn process_requests(&mut self) {
        let was_empty = self.write_buffer.is_empty();

        while let Some(cmd) = self.session_rx.try_pop() {
            if self.write_buffer.len() + cmd.body.len() + 64 > MAX_BUFFER_SIZE {
                break;
            }

            let msg = write_fix_message(
                cmd.msg_type,
                &self.outbound_sequence_number,
                &self.sender_comp_id,
                &self.target_comp_id,
                &cmd.body,
            );

            self.write_buffer.extend_from_slice(&msg);

            self.outbound_sequence_number = self.outbound_sequence_number.wrapping_add(1);
        }

        if was_empty && !self.write_buffer.is_empty() {
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
