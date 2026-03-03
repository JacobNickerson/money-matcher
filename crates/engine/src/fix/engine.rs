use crate::fix::session::FIXCommand;
use crate::lob::order::Order;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use ringbuf::{HeapProd, traits::*};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;

use crate::fix::session::Session;

const SERVER: Token = Token(0);

pub struct FixEngine {
    sessions: HashMap<Token, Session>,
    listener: TcpListener,
    tx: HeapProd<Order>,
    poll: Poll,
    token_counter: usize,
}

impl FixEngine {
    pub fn new(addr: SocketAddr, tx: HeapProd<Order>) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let poll = Poll::new()?;
        let mut this = Self {
            sessions: HashMap::new(),
            listener,
            tx,
            poll,
            token_counter: 2,
        };
        this.poll
            .registry()
            .register(&mut this.listener, SERVER, Interest::READABLE)?;
        Ok(this)
    }

    fn register_session(&mut self, mut stream: TcpStream) -> io::Result<()> {
        self.poll.registry().register(
            &mut stream,
            Token(self.token_counter),
            Interest::READABLE,
        )?;
        self.sessions
            .insert(Token(self.token_counter), Session::new(stream));
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
    use super::*;
    use std::thread;

    #[test]
    #[ignore]
    fn mpsc_test() {
        let (mut prod, mut cons) = ringbuf::HeapRb::<Order>::new(256).split();
        let addr: SocketAddr = "127.0.0.1:34254".parse().unwrap();
        let engine_thread = thread::spawn(move || {
            let mut _engine = FixEngine::new(addr, prod).unwrap();
            _engine.run();
        });
        loop {
            if let Some(order) = cons.try_pop() {
                println!("Read Order | {:?} |", order,);
                break;
            }
        }
        engine_thread.join().unwrap();
    }
}
