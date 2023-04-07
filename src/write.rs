use std::io;
use std::io::Write;

macro_rules! serialize_io_num {
    ($($t:ty),*) => {
        $(
        impl SerializeIo for $t {
            fn serialize<W>(&self, mut w: W) -> io::Result<()>
            where W: Write
            {
                w.write_all(&self.to_le_bytes())
            }
        })*
    }
}

macro_rules! write_dyn_self_impl {
    ($t:ty) => {
        impl<T> SerializeIo for $t
        where
            T: SerializeIo,
        {
            fn serialize<W>(&self, mut w: W) -> io::Result<()>
            where
                W: Write,
            {
                w.write_dyn(self)
            }
        }
    };
}

/// An extension to the standard [Write] trait.
pub trait ToraWrite {
    /// Serialize and write the given data.
    fn writes<S>(&mut self, s: &S) -> io::Result<()>
    where
        S: SerializeIo;

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
///     fn serialize<W>(&self, w: W) -> io::Result<()>
///     where
///         W: Write;
/// }
///
/// impl SerializeIo for i32 {
///     fn serialize<W>(&self, mut w: W) -> io::Result<()>
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
    fn serialize<W>(&self, w: W) -> io::Result<()>
    where
        W: Write;
}

serialize_io_num!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, usize);

impl SerializeIo for char {
    fn serialize<W>(&self, w: W) -> io::Result<()>
    where
        W: Write,
    {
        (*self as u32).serialize(w)
    }
}

impl SerializeIo for bool {
    fn serialize<W>(&self, w: W) -> io::Result<()>
    where
        W: Write,
    {
        (*self as u8).serialize(w)
    }
}

impl SerializeIo for String {
    fn serialize<W>(&self, w: W) -> io::Result<()>
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
    fn serialize<W>(&self, mut w: W) -> io::Result<()>
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

impl<T, const N: usize> SerializeIo for [T; N]
where
    T: SerializeIo,
{
    fn serialize<W>(&self, mut w: W) -> io::Result<()>
    where
        W: Write,
    {
        for t in self {
            w.writes(t)?;
        }
        Ok(())
    }
}

write_dyn_self_impl!(&[T]);
write_dyn_self_impl!(Vec<T>);
