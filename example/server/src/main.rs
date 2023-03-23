use std::io;
use std::net::TcpListener;

use tora::write::ToraWrite;
use tora::SerializeIo;

#[derive(SerializeIo)]
struct Message {
    sender: String,
    content: String,
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:12345")?;
    let (mut conn, _) = listener.accept()?;

    let message = Message {
        sender: "John".to_string(),
        content: "Hello, world!".to_string(),
    };

    conn.writes(&message)
}
