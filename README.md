<div align="center">
    <h1>Tora</h1>
    <p>A lite, byte-based serialization and deserialization library</p>
    <hr>
    <sub>*A zero-serde solution*</sub>
</div>

## Examples

### Network data transfer

#### Host

```rust
use std::io;
use std::net::TcpListener;

use tora::write::ToraWrite;
use tora::WriteStruct;

#[derive(WriteStruct)]
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
```

#### Client

```rust
use std::io;
use std::net::TcpStream;

use tora::read::ToraRead;
use tora::ReadStruct;

#[derive(ReadStruct)]
struct Message {
    sender: String,
    content: String,
}

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:12345")?;
    let Message { sender, content } = stream.reads()?;
    println!("{}: {}", sender, content);
    Ok(())
}
```
