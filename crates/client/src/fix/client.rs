use netlib::fix_core::{messages::FixMessage, types::FixFrame};
use nexus_queue::{Full, spsc};
use std::thread;

use crate::fix::session::Session;

pub struct FixClient {
    session_tx: spsc::Producer<FixFrame>,
}

impl FixClient {
    pub fn start() -> Self {
        let (session_tx, mut session_rx) = spsc::ring_buffer::<FixFrame>(8192);

        thread::spawn(move || {
            let mut session = Session::connect().expect("FIX connect failed");

            loop {
                if let Some(frame) = session_rx.pop() {
                    session.send_message(frame.msg_type, frame.body);
                } else {
                    std::hint::spin_loop();
                }
            }
        });

        Self { session_tx }
    }

    pub fn push_command<T>(&mut self, cmd: T)
    where
        T: FixMessage,
    {
        loop {
            let mut frame = FixFrame {
                msg_type: T::MESSAGE_TYPE,
                body: cmd.as_bytes(),
            };

            match self.session_tx.push(frame) {
                Ok(_) => break,
                Err(Full(c)) => {
                    frame = c;
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
    #[ignore]
    fn test() {
        let mut engine = FixClient::start();
        engine.push_command(NewOrder::new(
            1,
            1,
            10,
            2,
            666,
            1,
            "OSISTRING".to_string(),
            0,
            "OPT".to_string(),
        ));
        thread::sleep(Duration::from_secs(10));
    }
}
