//! Stream that reads from a slice of bytes.
use crate::{BinaryError, BinaryResult, ReadStream, SeekStream};
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

/// Stream that wraps a slice of bytes.
pub struct SliceStream<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> SliceStream<'a> {
    /// Create a slice stream.
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(buffer),
        }
    }
}

impl SeekStream for SliceStream<'_> {
    fn seek(&mut self, to: u64) -> BinaryResult<u64> {
        Ok(self.cursor.seek(SeekFrom::Start(to))?)
    }

    fn tell(&mut self) -> BinaryResult<u64> {
        Ok(self.cursor.stream_position()?)
    }

    fn len(&self) -> BinaryResult<usize> {
        Ok(self.cursor.get_ref().len())
    }
}

impl Read for SliceStream<'_> {
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

impl ReadStream for SliceStream<'_> {}
