<div align="center">
    <h1>Tora</h1>
    <p>A lite, byte-based serialization and deserialization library</p>
</div>

*A zero-serde solution*

---

## Examples

### Network data transfer

```rust
use std::io;
use std::net::TcpListener;

use tora::write::ToraWrite;
use tora::SerializeIo;

#[derive(SerializeIo)] // tora-provided
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

    conn.writes(&message) // tora-provided
}
```

### Data storage

```rust
use std::io;

use tora::{FromReader, SerializeIo};

#[derive(FromReader, SerializeIo, Debug)]
struct Employee {
    name: String,
    age: i32,
}

fn main() -> io::Result<()> {
    // 34 bytes.
    tora::write_to_file(
        "EmployeeList.tora",
        &[
            Employee {
                name: "John Doe".to_string(),
                age: 21,
            },
            Employee {
                name: "Walter White".to_string(),
                age: 52,
            },
        ]
        .as_slice(),
    )?;

    let employees: Vec<Employee> = tora::read_from_file("EmployeeList.tora")?;
    println!("{:#?}", employees);
    Ok(())
}
```
