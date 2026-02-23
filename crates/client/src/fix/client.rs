use netlib::fix_core::messages::{IntoBytes, get_maturity_month_year, get_timestamp};
use nexus_queue::{Full, spsc};
use std::thread;

use crate::fix::session::Session;

pub struct FixClient {
    session_tx: spsc::Producer<Vec<u8>>,
}

impl FixClient {
    pub fn start() -> Self {
        let (session_tx, mut session_rx) = spsc::ring_buffer::<Vec<u8>>(8192);

        thread::spawn(move || {
            let mut session = Session::connect().expect("FIX connect failed");

            loop {
                if let Some(cmd) = session_rx.pop() {
                    session.send_message(b"D", cmd);
                } else {
                    std::hint::spin_loop();
                }
            }
        });

        Self { session_tx }
    }

    pub fn push_command<T>(&mut self, cmd: T)
    where
        T: IntoBytes,
    {
        loop {
            let mut message = cmd.as_bytes();
            match self.session_tx.push(message) {
                Ok(_) => break,
                Err(Full(c)) => {
                    message = c;
                    std::hint::spin_loop();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use netlib::fix_core::types::NewOrder;

    use super::*;
    use std::time::Duration;

    #[test]
    fn test() {
        let mut engine = FixClient::start();
        engine.push_command(NewOrder {
            cl_ord_id: 1,
            qty: 10,
            price: 10,
            side: 0,
            symbol: *b"XYZ",
        });
        thread::sleep(Duration::from_secs(10));
    }
}
