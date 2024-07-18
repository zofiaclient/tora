use std::io;
use std::io::Write;

macro_rules! serialize_io_num {
    ($($t:ty),*) => {
        $(
        impl SerializeIo for $t {
            fn serialize<W>(&self, w: &mut W) -> io::Result<()>
            where W: Write
            {
                w.write_all(&self.to_le_bytes())
            }
        })*
    }
}

/// An extension to the standard [Write] trait.
pub trait ToraWrite {
    /// Serialize and write the given data.
    fn writes<S>(&mut self, s: &S) -> io::Result<()>
    where
        S: SerializeIo;
}

impl<W> ToraWrite for W
where
    W: Write,
{
    fn writes<S>(&mut self, s: &S) -> io::Result<()>
    where
        S: SerializeIo,
    {
        s.serialize(self)
    }
}

/// A trait marking a type as capable of serializing itself to a writer.
///
/// ```
/// use std::io;
/// use std::io::Write;
///
/// pub trait SerializeIo {
///     fn serialize<W>(&self, w: &mut W) -> io::Result<()>
///     where
///         W: Write;
/// }
///
/// impl SerializeIo for i32 {
///     fn serialize<W>(&self, w: &mut W) -> io::Result<()>
///     where W: Write
///     {
///         w.write_all(&self.to_le_bytes())
///     }
/// }
/// ```
pub trait SerializeIo {
    /// Serialize this type into the given writer.
    ///
    /// Implementations should call `write_all`.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write;
}

serialize_io_num!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, usize);

impl SerializeIo for char {
    /// Serializes this char as a u32.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        (*self as u32).serialize(w)
    }
}

impl SerializeIo for bool {
    /// Serializes this bool as a u8.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        (*self as u8).serialize(w)
    }
}

impl SerializeIo for () {
    /// Immediately returns [Ok] of unit value.
    fn serialize<W>(&self, _w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        Ok(())
    }
}

impl<T, Z> SerializeIo for (T, Z)
where
    T: SerializeIo,
    Z: SerializeIo,
{
    /// Writes a tuple of [T] and [Z], respectively.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.writes(&self.0)?;
        w.writes(&self.1)
    }
}

impl<T, Z, H> SerializeIo for (T, Z, H)
where
    T: SerializeIo,
    Z: SerializeIo,
    H: SerializeIo,
{
    /// Writes a tuple of [T], [Z], and [H], respectively.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.writes(&self.0)?;
        w.writes(&self.1)?;
        w.writes(&self.2)
    }
}

impl SerializeIo for String {
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.as_str().serialize(w)
    }
}

impl<'a> SerializeIo for &'a str {
    /// Write the given string in UTF-8.
    ///
    /// If the given string does not end in a NUL `0x00` byte, one will be appended.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.write_all(self.as_bytes())?;

        if !self.ends_with(0u8 as char) {
            w.write_all(&[0])?;
        }
        Ok(())
    }
}

impl<T> SerializeIo for Option<T>
where
    T: SerializeIo,
{
    /// If this Option is Some, writes true and the inner value, else false.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.writes(&self.is_some())?;

        if let Some(ref v) = self {
            w.writes(v)?;
        }
        Ok(())
    }
}

impl<T, E> SerializeIo for Result<T, E>
where
    T: SerializeIo,
    E: SerializeIo,
{
    /// If this Result is an error, writes true and the inner error, else false and the inner value.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.writes(&self.is_err())?;

        match self {
            Ok(v) => w.writes(v),
            Err(v) => w.writes(v),
        }
    }
}

impl<T, const N: usize> SerializeIo for [T; N]
where
    T: SerializeIo,
{
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        for t in self {
            w.writes(t)?;
        }
        Ok(())
    }
}

macro_rules! dyn_impl {
    ($t: ty) => {
        #[cfg(feature = "dyn_impl")]
        impl<T> SerializeIo for $t
        where
            T: SerializeIo,
        {
            fn serialize<W>(&self, w: &mut W) -> io::Result<()>
            where
                W: Write,
            {
                w.writes(&(self.len() as u32))?;

                for obj in self.iter() {
                    w.writes(obj)?;
                }
                Ok(())
            }
        }
    };
}

dyn_impl!(&[T]);
dyn_impl!(Vec<T>);
