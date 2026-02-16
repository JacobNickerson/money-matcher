use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_basic() -> std::io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:34254")?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut reader = BufReader::new(stream.try_clone()?);
                    let mut line = String::new();
                    reader.read_line(&mut line)?;
                    println!("Received: {}", line);

                    let mut writer = BufWriter::new(stream);
                    writer.write(b"Hello, client!\n")?;
                    writer.flush().unwrap();
                }
                Err(e) => println!("Connection error: {}", e),
            }
        }
        Ok(())
    }
}
