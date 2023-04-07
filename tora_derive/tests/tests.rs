use std::fmt::Debug;
use std::io;
use std::io::{Read, Write};

use tora::read::{FromReader, ToraRead};
use tora::write::{SerializeIo, ToraWrite};
use tora_derive::{ReadEnum, ReadStruct, WriteEnum, WriteStruct};

#[derive(Default)]
struct MockStream {
    inner: Vec<u8>,
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Read for MockStream {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let mut i = 0;

        while i < buf.len() {
            match self.inner.get(i) {
                Some(b) => buf.write_all(&[*b])?,
                None => break,
            }
            i += 1;
        }
        self.inner = self.inner[i..].to_vec();
        Ok(i)
    }
}

#[derive(ReadStruct, WriteStruct, Debug, PartialEq)]
struct TuplePacket(u8, String, Vec<u8>);

#[derive(ReadStruct, WriteStruct, Debug, PartialEq)]
struct NamedPacket {
    id: u8,
    sender: String,
    content: Vec<u8>,
}

#[derive(WriteEnum, ReadEnum, Debug, PartialEq)]
#[type_variant_id(i64)]
enum EnumPacket {
    Ping,
    PlayerJoin(u8, String),
    PlayerMove {
        player_id: u8,
        destination: [f64; 3],
    },
}

fn mock_stream_rw_test<T>(data: T) -> io::Result<()>
where
    T: SerializeIo + FromReader + PartialEq + Debug,
{
    let mut stream = MockStream::default();
    stream.writes(&data)?;

    let received = stream.reads()?;
    assert_eq!(data, received);
    Ok(())
}

#[test]
fn tuple_packet() -> io::Result<()> {
    mock_stream_rw_test(TuplePacket(5, "John".to_string(), vec![1, 2, 3]))
}

#[test]
fn named_packet() -> io::Result<()> {
    mock_stream_rw_test(NamedPacket {
        id: 5,
        sender: "John".to_string(),
        content: vec![1, 2, 3],
    })
}

#[test]
fn enum_packet() -> io::Result<()> {
    mock_stream_rw_test(EnumPacket::PlayerMove {
        player_id: 5,
        destination: [1.4, 3.1, 9.0],
    })
}
