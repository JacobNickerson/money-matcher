use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::net::TcpStream;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_basic() -> std::io::Result<()> {
        let mut stream = TcpStream::connect("127.0.0.1:34254")?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = BufWriter::new(stream);
        writer.write(b"Hello, engine!\n")?;
        writer.flush().unwrap();

        let mut line = String::new();
        reader.read_line(&mut line)?;
        println!("Received: {}", line);
        Ok(())
    }
}
