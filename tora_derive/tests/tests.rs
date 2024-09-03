use std::fmt::Debug;
use std::io;
use std::io::Cursor;

use tora::read::{FromReader, ToraRead};
use tora::write::{SerializeIo, ToraWrite};
use tora_derive::{ReadEnum, ReadStruct, WriteEnum, WriteStruct};

#[derive(Debug, PartialEq, ReadStruct, WriteStruct)]
struct StructPacket {
    id: u8,
    sender: String,
    content: Vec<u8>,
}

#[derive(Debug, PartialEq, ReadStruct, WriteStruct)]
struct TuplePacket(u8, Result<(), String>, Vec<u8>);

#[derive(Debug, PartialEq, ReadEnum, WriteEnum)]
#[type_variant_id(i64)]
enum EnumPacket {
    Ping,
    PlayerJoin(PlayerJoin),
    PlayerMove {
        player_id: u8,
        destination: [f64; 3],
        feet_position: (f64, f64, f64),
    },
}

#[derive(Debug, PartialEq, ReadStruct, WriteStruct)]
struct PlayerJoin {
    id: u8,
    name: Option<String>,
}

fn assert_rw_eq<T>(data: T) -> io::Result<()>
where
    T: SerializeIo + FromReader + PartialEq + Debug,
{
    let mut bytes = Vec::new();
    bytes.writes(&data)?;

    let mut cursor = Cursor::new(bytes);
    let received = cursor.reads()?;

    assert_eq!(data, received);
    Ok(())
}

#[test]
fn struct_packet() -> io::Result<()> {
    assert_rw_eq(StructPacket {
        id: 5,
        sender: "John".to_string(),
        content: vec![1, 2, 3],
    })
}

#[test]
fn tuple_packet() -> io::Result<()> {
    assert_rw_eq(TuplePacket(
        5,
        Err("Could not get username".to_string()),
        vec![1, 2, 3],
    ))
}

#[test]
fn enum_packet() -> io::Result<()> {
    assert_rw_eq(EnumPacket::PlayerMove {
        player_id: 5,
        destination: [1.4, 3.1, 9.0],
        feet_position: (1.4, 3.1, 7.0),
    })?;
    assert_rw_eq(EnumPacket::PlayerJoin(PlayerJoin {
        id: 1,
        name: Some("Joseph".to_string()),
    }))
}

#[test]
fn boxes() -> io::Result<()> {
    assert_rw_eq(Box::new(EnumPacket::Ping))
}
