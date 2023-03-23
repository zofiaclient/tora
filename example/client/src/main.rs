use std::io;
use std::net::TcpStream;

use tora::read::ToraRead;
use tora::FromReader;

#[derive(FromReader)]
struct Message {
    sender: String,
    content: String,
}

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:12345")?;
    let Message { sender, content } = stream.reads()?;
    println!("{sender}: {content}");
    Ok(())
}
