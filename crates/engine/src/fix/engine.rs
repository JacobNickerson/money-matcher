use crate::fix::session::{FIXCommand, FIXReply};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token, Waker};
use ringbuf::{HeapCons, HeapProd, traits::*};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::fix::session::Session;

const SERVER: Token = Token(0);
const WAKE: Token = Token(1);

pub struct FixEngine {
    sessions: HashMap<Token, Session>,
    listener: TcpListener,
    tx: HeapProd<FIXCommand>,
    rx: HeapCons<FIXReply>,
    waker: Arc<Waker>,
    poll: Poll,
    token_counter: usize,
}

impl FixEngine {
    pub fn new(
        addr: SocketAddr,
        tx: HeapProd<FIXCommand>,
        rx: HeapCons<FIXReply>,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let poll = Poll::new()?;
        let waker = { Arc::new(Waker::new(poll.registry(), WAKE)?) };
        let mut this = Self {
            sessions: HashMap::new(),
            listener,
            tx,
            rx,
            waker,
            poll,
            token_counter: 100,
        };
        this.poll
            .registry()
            .register(&mut this.listener, SERVER, Interest::READABLE)?;
        Ok(this)
    }

    pub fn get_waker(&self) -> Arc<Waker> {
        self.waker.clone()
    }

    fn register_session(&mut self, mut stream: TcpStream) -> io::Result<()> {
        self.poll.registry().register(
            &mut stream,
            Token(self.token_counter),
            Interest::READABLE,
        )?;
        self.sessions.insert(
            Token(self.token_counter),
            Session::new(Token(self.token_counter), stream),
        );
        self.token_counter += 1;
        Ok(())
    }

    pub fn run(&mut self) {
        let mut events = Events::with_capacity(1024);
        println!("Server running on {}", self.listener.local_addr().unwrap());
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        if let Ok((new_stream, _)) = self.listener.accept() {
                            self.register_session(new_stream).unwrap();
                        }
                    }
                    WAKE => {
                        while let Some(reply) = self.rx.try_pop() {
                            if let Some(session) = self.sessions.get_mut(&reply.token) {
                                println!("CALLING HANDLE REPLY");
                                session.handle_reply(reply.data);
                                self.poll
                                    .registry()
                                    .reregister(
                                        &mut session.stream,
                                        reply.token,
                                        Interest::READABLE | Interest::WRITABLE,
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    token if event.is_writable() => {
                        if let Some(session) = self.sessions.get_mut(&token) {
                            println!("CALLING FLUSH");
                            session.flush();

                            if session.write_buffer.is_empty() {
                                self.poll
                                    .registry()
                                    .reregister(&mut session.stream, token, Interest::READABLE)
                                    .unwrap();
                            }
                        }
                    }
                    token if event.is_readable() => {
                        if let Some(session) = self.sessions.get_mut(&token)
                            && let Err(e) = session.poll(&mut self.tx)
                        {
                            eprintln!("Error polling session: {}", e);
                            self.sessions.remove(&token);
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
    use netlib::fix_core::messages::execution_report::ExecutionReport;

    use super::*;
    use crate::fix::session::FIXCommand;
    use std::thread;

    #[test]
    #[ignore]
    fn mpsc_test() {
        let (mut prod, mut cons) = ringbuf::HeapRb::<FIXCommand>::new(256).split();

        let (mut reply_prod, mut reply_cons) = ringbuf::HeapRb::<FIXReply>::new(256).split();

        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let mut engine = FixEngine::new(addr, prod, reply_cons).unwrap();
        let waker = engine.get_waker();
        let engine_thread = thread::spawn(move || {
            engine.run();
        });

        loop {
            if let Some(cmd) = cons.try_pop() {
                match cmd {
                    FIXCommand::Order(token, order) => {
                        println!("Read Order | {:?} | {:?} |", token, order,);

                        let report = ExecutionReport {};
                        let reply = FIXReply {
                            token,
                            data: report,
                        };

                        reply_prod.try_push(reply).unwrap();
                        waker.wake().unwrap();

                        break;
                    }
                }
            }
        }
        engine_thread.join().unwrap();
    }
}
