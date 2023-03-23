use std::collections::HashMap;
use std::{io, thread};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use tora::write::ToraWrite;

static USERS: Arc<Mutex<HashMap<String, TcpStream>>> = Arc::new(Mutex::default());

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:12345")?;
    
    thread::spawn(move || {
        let (conn, addr) = listener.accept()?;
        
    })
}
