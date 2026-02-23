use std::{net::TcpListener, thread};

use crate::fix::session::Session;

pub struct FixEngine;

impl FixEngine {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:34254").expect("bind failed");

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    thread::spawn(move || {
                        let mut connection = Session::new(stream);
                        connection.run();
                    });
                }
            }
        });

        Self
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test() {
        let _engine = FixEngine::start();
        thread::sleep(Duration::from_secs(10));
    }
}
