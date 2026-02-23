use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

#[cfg(test)]
mod tests {
    use std::{io, net::SocketAddr, thread};

    use super::*;

    #[test]
    fn test_basic() -> io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:34254")?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        let mut reader = BufReader::new(stream.try_clone().unwrap());

                        loop {
                            let mut buf = [0u8; 4096];
                            let n = reader.read(&mut buf).unwrap();
                            println!("{:x?}", &buf[..n]);
                        }
                    });
                }
                Err(e) => println!("{}", e),
            }
        }

        Ok(())
    }
}
