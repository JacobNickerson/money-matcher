use std::io::BufReader;
use std::io::BufWriter;
use std::io::Result;
use std::io::prelude::*;
use std::net::TcpStream;

#[cfg(test)]
mod tests {

    use {std::thread, std::time::Duration};

    use super::*;

    fn test_basic() -> Result<()> {
        let mut stream = TcpStream::connect("127.0.0.1:34254")?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = BufWriter::new(stream);

        for i in 1..=10 {
            writer.write(b"Hello, engine 1!\n")?;
            writer.flush().unwrap();
        }

        writer.write(b"FINISH\n")?;
        writer.flush().unwrap();

        Ok(())
    }

    fn test_basic2() -> Result<()> {
        let mut stream = TcpStream::connect("127.0.0.1:34254")?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = BufWriter::new(stream);

        for i in 1..=10 {
            writer.write(b"Hello, engine 2!\n")?;
            writer.flush().unwrap();
        }

        writer.write(b"FINISH\n")?;
        writer.flush().unwrap();

        Ok(())
    }

    #[test]
    fn test_both() {
        let t1 = thread::spawn(|| test_basic().unwrap());
        let t2 = thread::spawn(|| test_basic2().unwrap());

        t1.join().unwrap();
        t2.join().unwrap();
    }
}
