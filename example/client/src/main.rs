use std::io;
use std::net::TcpStream;
use tora::read::ReadExt;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:12345")?;
    let message = stream.read_string()?;
    println!("{message}");
    Ok(())
}
