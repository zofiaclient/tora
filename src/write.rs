use std::io;
use std::io::Write;

/// An extension to the standard [Write] trait.
pub trait ToraWrite {
    /// Serialize and write the given data.
    fn writes<S>(&mut self, s: &S) -> io::Result<()>
    where
        S: SerializeIo;

    /// Write the given string in UTF-8.
    ///
    /// If the given string does not end in a NUL `0x00` byte, one will be appended.
    fn write_utf8<S>(&mut self, s: S) -> io::Result<()>
    where
        S: AsRef<str>;

    /// Write a dynamic amount of bytes.
    ///
    /// Opposite of [ToraRead::read_dyn_bytes](crate::read::ToraRead::read_dyn_bytes).
    fn write_dyn_bytes<B>(&mut self, b: B) -> io::Result<()>
    where
        B: AsRef<[u8]>;

    /// Write a dynamic amount of objects.
    ///
    /// Opposite of [ToraRead::read_dyn](crate::read::ToraRead::read_dyn).
    fn write_dyn<T, D>(&mut self, d: D) -> io::Result<()>
    where
        D: AsRef<[T]>,
        T: SerializeIo;
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

    fn write_utf8<S>(&mut self, msg: S) -> io::Result<()>
    where
        S: AsRef<str>,
    {
        let s = msg.as_ref();
        self.write_all(s.as_bytes())?;

        if !s.ends_with(0u8 as char) {
            self.write_all(&[0])?;
        }
        Ok(())
    }

    fn write_dyn_bytes<B>(&mut self, b: B) -> io::Result<()>
    where
        B: AsRef<[u8]>,
    {
        let b = b.as_ref();
        self.writes(&(b.len() as u32))?;
        self.write_all(b)
    }

    fn write_dyn<T, D>(&mut self, d: D) -> io::Result<()>
    where
        D: AsRef<[T]>,
        T: SerializeIo,
    {
        let d = d.as_ref();
        self.writes(&(d.len() as u32))?;

        for obj in d {
            self.writes(obj)?;
        }
        Ok(())
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
    /// Should call `write_all`.
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write;
}

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

serialize_io_num!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, usize);

impl SerializeIo for char {
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        (*self as u32).serialize(w)
    }
}

impl SerializeIo for bool {
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        (*self as u8).serialize(w)
    }
}

impl SerializeIo for String {
    /// Equivalent to [ToraWrite::write_utf8].
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.write_utf8(self)
    }
}

impl<'a> SerializeIo for &'a str {
    /// Equivalent to [ToraWrite::write_utf8].
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.write_utf8(self)
    }
}

impl<T> SerializeIo for &[T]
where
    T: SerializeIo,
{
    /// Equivalent to [ToraWrite::write_dyn].
    fn serialize<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.write_dyn(self)
    }
}
