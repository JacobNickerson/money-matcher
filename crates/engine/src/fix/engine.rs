use std::{net::TcpListener, thread};

use crate::fix::session::FIXCommand;
use nexus_queue::mpsc::Producer;

use crate::fix::session::Session;

pub struct FixEngine;

impl FixEngine {
    pub fn start(lob_tx: Producer<FIXCommand>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:34254").expect("bind failed");

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let tx = lob_tx.clone();

                    thread::spawn(move || {
                        let mut connection = Session::new(stream, tx);
                        connection.run();
                    });
                }
            }
        });

        Self {}
    }
}

#[cfg(test)]
mod tests {
    use crate::fix::session::FIXCommand;
    use nexus_queue::mpsc;
    use std::time::Duration;

    use super::*;

    #[test]
    fn mpsc_test() {
        let (lob_tx, mut lob_rx) = mpsc::bounded::<FIXCommand>(1024);

        let _engine = FixEngine::start(lob_tx);
        loop {
            if let Some(cmd) = lob_rx.pop() {
                match cmd {
                    FIXCommand::Order(order) => {
                        println!("Read Order | {:?} |", order,);
                        break;
                    }
                }
            }
        }
    }
}
