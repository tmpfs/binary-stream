//! Stream that reads from and writes to an owned buffer.
use crate::{BinaryError, BinaryResult, ReadStream, SeekStream, WriteStream};
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};

/// Stream that wraps an owned buffer.
pub struct MemoryStream {
    cursor: Cursor<Vec<u8>>,
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
    fn seek(&mut self, to: usize) -> BinaryResult<usize> {
        let position = self.cursor.seek(SeekFrom::Start(to as u64))?;
        Ok(position as usize)
    }

    fn tell(&mut self) -> BinaryResult<usize> {
        Ok(self.cursor.stream_position()? as usize)
    }

    fn len(&self) -> BinaryResult<usize> {
        Ok(self.cursor.get_ref().len())
    }
}

impl Read for MemoryStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.cursor.position() as usize + buffer.len() > self.cursor.get_ref().len() {
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

impl Into<Vec<u8>> for MemoryStream {
    fn into(self) -> Vec<u8> {
        self.cursor.into_inner()
    }
}

impl ReadStream for MemoryStream {}
impl WriteStream for MemoryStream {}
