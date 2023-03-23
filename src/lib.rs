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
