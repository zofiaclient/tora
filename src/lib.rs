//! # tora
//!
//! Tora is a byte-based serialization and deserialization library.
//!
//! ```
//! use std::io;
//! use std::io::Cursor;
//!
//! use tora::{ReadEnum, ReadStruct, WriteEnum, WriteStruct};
//! use tora::read::ToraRead;
//! use tora::write::ToraWrite;
//!
//! #[derive(Debug, PartialEq, ReadEnum, WriteEnum)]
//! #[type_variant_id(i64)]
//! enum Packet {
//!     Ping,
//!     PlayerJoin(PlayerJoin),
//!     PlayerMove {
//!         id: u8,
//!         destination: [f64; 3],
//!     },
//! }
//!
//! #[derive(Debug, PartialEq, ReadStruct, WriteStruct)]
//! struct PlayerJoin {
//!     id: u8,
//!     username: Option<String>
//! }
//!
//! fn main() -> io::Result<()> {
//!     let se = Packet::PlayerMove { id: 5, destination: [1.1, 2.4, 3.1] };
//!
//!     let mut bytes = Vec::new();
//!     bytes.writes(&se)?;
//!
//!     let mut cursor = Cursor::new(bytes);
//!     let de = cursor.reads()?;
//!
//!     assert_eq!(se, de);
//!     Ok(())
//! }
//! ```

use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(feature = "tora_derive")]
pub use tora_derive::*;

use crate::read::{FromReader, ToraRead};
use crate::write::{SerializeIo, ToraWrite};

pub mod read;
pub mod write;

/// Serialize the content and write it to the file at the given path.
pub fn write_to_file<P, C>(path: P, content: &C) -> io::Result<()>
where
    P: AsRef<Path>,
    C: SerializeIo,
{
    let mut file = File::create(path)?;
    file.writes(content)
}

/// Try to deserialize [T] from the file at the given path.
pub fn read_from_file<T, P>(path: P) -> io::Result<T>
where
    P: AsRef<Path>,
    T: FromReader,
{
    let mut file = File::open(path)?;
    file.reads()
}
