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

/// A reader that reads and discards 
#[derive(Default)]
pub struct PaddedReader {
    padding: usize,
}

impl PaddedReader {
    /// Reads and discards the amount of padding, then reads [T], and applies the new padding to
    /// future reads.
    pub fn reads_then_set_padding<T, R>(&mut self, r: &mut R, new_padding: usize) -> io::Result<T>
    where
        T: FromReader,
        R: Read,
    {
        let data = self.reads(r)?;
        self.padding = new_padding;
        Ok(data)
    }

    /// Reads and discards the amount of padding, then reads [T].
    pub fn reads<T, R>(&self, r: &mut R) -> io::Result<T>
    where
        T: FromReader,
        R: Read,
    {
        let mut temp = vec![0; self.padding];
        r.read_exact(&mut temp)?;
        r.reads()
    }

    /// Constructs a PaddedReader with the given initial padding.
    pub fn set_padding(&mut self, padding: usize) -> &mut Self {
        self.padding = padding;
        self
    }

    /// Constructs a PaddedReader with the given initial padding.
    pub const fn with_padding(padding: usize) -> Self {
        Self { padding }
    }

    /// Returns the current amount of padding this reader uses.
    pub const fn padding(&self) -> usize {
        self.padding
    }
}

/// Marks a type as able to be deserialized from a reader.
///
/// If you are implementing this trait, make sure tora's derive macros are inapplicable to your use
/// case.
///
/// # Examples
///
/// ```
/// use std::io;
/// use std::io::Read;
///
/// use tora::read::{FromReader, ToraRead};
///
/// struct CustomVec {
///     extended_capacity: u32,
///     content: Vec<u8>
/// }
///
/// impl FromReader for CustomVec {
///     fn from_reader<R>(r: &mut  R) -> io::Result<Self>
///     where
///         R: Read,
///     {
///         Ok(Self {
///             extended_capacity: r.reads()?,
///             content: r.reads()?
///         })
///     }
/// }
/// ```
pub trait FromReader: Sized {
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read;
}

from_reader_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, usize);

impl FromReader for bool {
    /// Reads a bool from this reader.
    ///
    /// Returns true if the read [u8] is **not** zero.
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        r.reads::<u8>().map(|x| x != 0)
    }
}

impl FromReader for char {
    /// Reads a character from this reader.
    ///
    /// Returns [ErrorKind::InvalidData] if the read [u32] cannot be converted to a [char].
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

impl<T> FromReader for Option<T>
where
    T: FromReader,
{
    /// Reads a bool and if true, reads and returns Some([T]).
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        if r.reads::<bool>()? {
            return Ok(Some(r.reads()?));
        }
        Ok(None)
    }
}

#[cfg(feature = "dyn_impl")]
impl<T> FromReader for Vec<T>
where
    T: FromReader,
{
    /// Reads a [u32], then reads N amount of [T] into a Vec and returns it.
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        let len = r.reads::<u32>()? as usize;
        let mut buf = Vec::with_capacity(len);

        for _ in 0..len {
            buf.push(r.reads()?);
        }
        Ok(buf)
    }
}

impl<T, const N: usize> FromReader for [T; N]
where
    T: FromReader + Copy + Default,
{
    /// Reads and deserializes [N] amount of [T].
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        let mut arr = [T::default(); N];

        for value in arr.iter_mut() {
            *value = r.reads()?;
        }
        Ok(arr)
    }
}

impl<T, E> FromReader for Result<T, E>
where
    T: FromReader,
    E: FromReader,
{
    /// Reads a boolean and if true, tries to deserialize the [E] type, else [T].
    fn from_reader<R>(r: &mut R) -> io::Result<Result<T, E>>
    where
        R: Read,
    {
        if r.reads()? {
            return Ok(Err(r.reads()?));
        }
        Ok(Ok(r.reads()?))
    }
}

impl FromReader for () {
    /// Immediately returns [Ok] of unit value.
    fn from_reader<R>(_r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        Ok(())
    }
}

impl<T, Z> FromReader for (T, Z)
where
    T: FromReader,
    Z: FromReader,
{
    /// Reads a tuple of [T] and [Z], respectively.
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        Ok((r.reads()?, r.reads()?))
    }
}

impl<T, Z, H> FromReader for (T, Z, H)
where
    T: FromReader,
    Z: FromReader,
    H: FromReader,
{
    /// Reads a tuple of [T], [Z], and [H], respectively.
    fn from_reader<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        Ok((r.reads()?, r.reads()?, r.reads()?))
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
    ///
    ///     let date = stream.reads::<u16>()?;
    ///     let employees: Vec<String> = stream.reads()?;
    ///
    ///     println!("{date}, {employees:?}");
    ///     Ok(())
    /// }
    /// ```
    fn reads<T>(&mut self) -> io::Result<T>
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
}
