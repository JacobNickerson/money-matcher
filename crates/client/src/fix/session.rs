use mio::{Registry, net::TcpStream};
use netlib::fix_core::{helpers::write_fix_message, messages::FixFrame};
use ringbuf::{HeapCons, traits::Consumer};
use std::{
    io::{self, Result, Write},
    net::SocketAddr,
};

use mio::{Events, Interest, Poll, Token, Waker};
use zerocopy::IntoBytes;
const NET: Token = Token(0);
const WAKE: Token = Token(1);

pub struct Session {
    pub inbound_sequence_number: u32,
    pub logged_in: bool,
    pub outbound_sequence_number: u32,
    pub sender_comp_id: String,
    pub stream: TcpStream,
    pub target_comp_id: String,
    pub write_buf: Vec<u8>,
    pub read_buf: Vec<u8>,
    pub session_rx: HeapCons<FixFrame>,
    pub poll: Poll,
}

impl Session {
    pub fn connect(addr: SocketAddr, poll: Poll, session_rx: HeapCons<FixFrame>) -> Result<Self> {
        let mut stream = TcpStream::connect(addr)?;

        {
            let registry = poll.registry();
            registry.register(&mut stream, NET, Interest::READABLE)?;
        }

        Ok(Self {
            inbound_sequence_number: 1,
            logged_in: false,
            outbound_sequence_number: 1,
            sender_comp_id: "CLIENT01".to_string(),
            stream,
            target_comp_id: "ENGINE01".to_string(),
            write_buf: Vec::new(),
            read_buf: Vec::new(),
            session_rx,
            poll,
        })
    }

    pub fn run(&mut self) {
        let mut events = Events::with_capacity(1024);
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    NET => {
                        if event.is_readable() {
                            //read
                        }

                        if event.is_writable() {
                            match self.stream.write(&self.write_buf) {
                                Ok(n) => {
                                    self.write_buf.drain(..n);

                                    if self.write_buf.is_empty() {
                                        let registry = self.poll.registry();
                                        registry
                                            .reregister(&mut self.stream, NET, Interest::READABLE)
                                            .unwrap();
                                    }
                                }
                                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                Err(e) => panic!("Error {}", e),
                            }
                        };
                    }
                    WAKE => {
                        if let Some(cmd) = self.session_rx.try_pop() {
                            write_fix_message(
                                &mut self.write_buf,
                                cmd.msg_type,
                                &self.outbound_sequence_number,
                                &self.sender_comp_id,
                                &self.target_comp_id,
                                &cmd.body,
                            );
                            self.outbound_sequence_number =
                                self.outbound_sequence_number.wrapping_add(1);

                            let registry = self.poll.registry();
                            registry
                                .reregister(
                                    &mut self.stream,
                                    NET,
                                    Interest::READABLE | Interest::WRITABLE,
                                )
                                .unwrap();
                            //stuff
                        }
                    }

                    _ => (),
                }
            }
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
    //    print_message(&session.write_buf);
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
    //        write_buf: Vec::new(),
    //    };
    //
    //    let body = Vec::new();
    //
    //    write_fix_message(
    //        &mut session.write_buf,
    //        &FIX_MESSAGE_TYPE_NEW_ORDER,
    //        &session.outbound_sequence_number,
    //        &session.sender_comp_id,
    //        &session.target_comp_id,
    //        &body,
    //    );
    //
    //    let s = from_utf8(&session.write_buf).expect("err");
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
