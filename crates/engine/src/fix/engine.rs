use std::{net::TcpListener, thread};

use netlib::fix_core::messages::FIXCommand;
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
    use netlib::fix_core::messages::FIXCommand;
    use nexus_queue::mpsc;
    use std::time::Duration;

    use super::*;

    #[test]
    fn mpsc_test() {
        let (lob_tx, mut lob_rx) = mpsc::bounded::<FIXCommand>(1024);

        let _engine = FixEngine::start(lob_tx);
        if let Some(cmd) = lob_rx.pop() {
            match cmd {
                FIXCommand::NewOrder(_s) => {
                    println!(
                        "Read New Order | cl_ord_id(11)={} | qty(38)={} | price(44)={} | side(54)={} | symbol(55)={}",
                        _s.cl_ord_id, _s.qty, _s.price, _s.side, _s.symbol
                    );
                }
            }
        }

        thread::sleep(Duration::from_secs(10));
    }
}
