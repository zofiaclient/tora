use std::io;
use std::io::{ErrorKind, Read};

macro_rules! from_reader_impl {
    ($($t:ty),*) => {
        $(
        impl FromReader for $t {
            fn from_reader<R>(r: &mut R) -> io::Result<Self>
            where
                R: Read,
            {
                let mut buf = [0; std::mem::size_of::<$t>()];
                r.read_exact(&mut buf).map(|_| <$t>::from_le_bytes(buf))
            }
        }
        )*
    };
}

/// ```
/// use std::io;
/// use std::net::TcpStream;
/// use tora::read::ToraRead;
///
/// fn main() -> io::Result<()> {
///     let mut stream = TcpStream::connect("127.0.0.1:12345")?;
///
///     let name = stream.read_utf8()?;
///     let age = stream.reads::<u32>()?;
///
///     println!("{name} is {age} years old.");
///     Ok(())
/// }
/// ```
pub trait FromReader: Sized {
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read;
}

from_reader_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, usize);

impl FromReader for bool {
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        r.reads::<u8>().map(|x| x != 0)
    }
}

impl FromReader for char {
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        r.reads::<u32>().and_then(|c| {
            char::from_u32(c)
                .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Not a character"))
        })
    }
}

impl FromReader for String {
    /// Read a UTF-8 string from this reader.
    ///
    /// Reads until a NUL `0x00` byte is encountered. Does not include the terminating byte.
    ///
    /// Returns [ErrorKind::InvalidData] if the received message is not valid UTF-8.
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        let mut buf = Vec::new();

        loop {
            let b = r.reads::<u8>()?;
            if b == 0 {
                break String::from_utf8(buf)
                    .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8"));
            }
            buf.push(b);
        }
    }
}

impl<T> FromReader for Vec<T>
where
    T: FromReader,
{
    /// Equivalent to [ToraRead::read_dyn].
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        r.read_dyn()
    }
}

/// An extension upon the standard [Read] implementation.
///
/// ```
/// use std::io;
/// use std::net::TcpStream;
/// use tora::read::ToraRead;
///
/// fn main() -> io::Result<()> {
///     let mut stream = TcpStream::connect("127.0.0.1:12345")?;
///     let message = stream.reads::<i32>()?;
///
///     println!("{}", message);
///     Ok(())
/// }
/// ```
pub trait ToraRead {
    /// Try to read and deserialize a type from this reader.
    ///
    /// ```
    /// use std::io;
    /// use std::net::TcpStream;
    /// use tora::read::ToraRead;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stream = TcpStream::connect("127.0.0.1:12345")?;
    ///     let message = stream.reads::<i32>()?;
    ///
    ///     println!("{}", message);
    ///     Ok(())
    /// }
    /// ```
    fn reads<T>(&mut self) -> io::Result<T>
    where
        T: FromReader;

    /// Read a dynamic amount of objects.
    ///
    /// Reads a [u32], then reads N amount of [T] into a Vec and returns it.
    fn read_dyn<T>(&mut self) -> io::Result<Vec<T>>
    where
        T: FromReader;
}

#[cfg(feature = "read_impl")]
impl<R> ToraRead for R
where
    R: Read,
{
    fn reads<T>(&mut self) -> io::Result<T>
    where
        T: FromReader,
    {
        T::from_reader(self)
    }

    fn read_dyn<T>(&mut self) -> io::Result<Vec<T>>
    where
        T: FromReader,
    {
        let len = self.reads::<u32>()? as usize;
        let mut buf = Vec::with_capacity(len);

        for _ in 0..len {
            buf.push(self.reads()?);
        }
        Ok(buf)
    }
}
