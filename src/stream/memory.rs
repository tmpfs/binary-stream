//! Stream that reads from and writes to an owned buffer.
use crate::{BinaryError, BinaryResult, ReadStream, SeekStream, WriteStream};
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};

/// Stream that wraps an owned buffer.
pub struct MemoryStream {
    cursor: Cursor<Vec<u8>>,
}

impl Default for MemoryStream {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStream {
    /// Create a memory stream.
    pub fn new() -> Self {
        Self {
            cursor: Cursor::new(Vec::new()),
        }
    }
}

impl SeekStream for MemoryStream {
    fn seek(&mut self, to: u64) -> BinaryResult<u64> {
        Ok(self.cursor.seek(SeekFrom::Start(to))?)
    }

    fn tell(&mut self) -> BinaryResult<u64> {
        Ok(self.cursor.stream_position()?)
    }

    fn len(&self) -> BinaryResult<u64> {
        Ok(self.cursor.get_ref().len() as u64)
    }
}

impl Read for MemoryStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.cursor.position() as usize + buffer.len()
            > self.cursor.get_ref().len()
        {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                BinaryError::ReadPastEof,
            ));
        }

        self.cursor.read(buffer)
    }
}

impl Write for MemoryStream {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.cursor.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl From<Vec<u8>> for MemoryStream {
    fn from(buffer: Vec<u8>) -> Self {
        MemoryStream {
            cursor: Cursor::new(buffer),
        }
    }
}

impl From<MemoryStream> for Vec<u8> {
    fn from(stream: MemoryStream) -> Self {
        stream.cursor.into_inner()
    }
}

impl ReadStream for MemoryStream {}
impl WriteStream for MemoryStream {}
