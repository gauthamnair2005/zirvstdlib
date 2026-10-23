//! Minimal I/O traits, modelled on the interfaces every Rust programmer
//! already knows but implemented without a hosted standard library.

use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

/// A source of bytes.
pub trait Read {
    fn read(&mut self, buffer: &mut [u8]) -> crate::Result<usize>;

    fn read_exact(&mut self, mut buffer: &mut [u8]) -> crate::Result<()> {
        while !buffer.is_empty() {
            let read = self.read(buffer)?;
            if read == 0 {
                return Err(crate::Error::UnexpectedEof);
            }
            buffer = &mut buffer[read..];
        }
        Ok(())
    }

    fn read_to_end(&mut self, output: &mut Vec<u8>) -> crate::Result<usize> {
        let mut total = 0;
        let mut chunk = [0u8; 512];
        loop {
            let read = self.read(&mut chunk)?;
            if read == 0 {
                return Ok(total);
            }
            output.extend_from_slice(&chunk[..read]);
            total += read;
        }
    }
}

/// A sink for bytes.
pub trait Write {
    fn write(&mut self, buffer: &[u8]) -> crate::Result<usize>;

    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, mut buffer: &[u8]) -> crate::Result<()> {
        while !buffer.is_empty() {
            let written = self.write(buffer)?;
            if written == 0 {
                return Err(crate::Error::WriteZero);
            }
            buffer = &buffer[written..];
        }
        Ok(())
    }
}

/// A seekable byte stream.
pub trait Seek {
    fn seek(&mut self, position: SeekFrom) -> crate::Result<u64>;
}

/// An in-memory cursor over a byte slice.
pub struct Cursor<T> {
    inner: T,
    position: usize,
}

impl<T: AsRef<[u8]>> Cursor<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buffer: &mut [u8]) -> crate::Result<usize> {
        let data = self.inner.as_ref();
        if self.position >= data.len() {
            return Ok(0);
        }
        let available = &data[self.position..];
        let count = buffer.len().min(available.len());
        buffer[..count].copy_from_slice(&available[..count]);
        self.position += count;
        Ok(count)
    }
}

impl<T: AsRef<[u8]>> Seek for Cursor<T> {
    fn seek(&mut self, position: SeekFrom) -> crate::Result<u64> {
        let length = self.inner.as_ref().len() as i64;
        let target = match position {
            SeekFrom::Start(value) => value as i64,
            SeekFrom::End(value) => length + value,
            SeekFrom::Current(value) => self.position as i64 + value,
        };
        if target < 0 {
            return Err(crate::Error::InvalidArgument);
        }
        self.position = target as usize;
        Ok(self.position as u64)
    }
}

impl Write for Vec<u8> {
    fn write(&mut self, buffer: &[u8]) -> crate::Result<usize> {
        self.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}

/// Copy every byte from `reader` to `writer`.
pub fn copy<R: Read + ?Sized, W: Write + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> crate::Result<u64> {
    let mut buffer = [0u8; 1024];
    let mut total = 0;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(total);
        }
        writer.write_all(&buffer[..read])?;
        total += read as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trip() {
        let mut cursor = Cursor::new(b"hello world".to_vec());
        let mut buffer = [0u8; 5];
        cursor.read_exact(&mut buffer).unwrap();
        assert_eq!(&buffer, b"hello");
        cursor.seek(SeekFrom::Start(6)).unwrap();
        let mut rest = Vec::new();
        cursor.read_to_end(&mut rest).unwrap();
        assert_eq!(rest, b"world");
    }
}
