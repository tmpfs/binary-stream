//! Stream for operating on files.
use crate::{BinaryError, BinaryResult, ReadStream, SeekStream, WriteStream};
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Read, SeekFrom, Write};

/// Stream that wraps a file.
pub struct FileStream(pub std::fs::File);

impl SeekStream for FileStream {
    fn seek(&mut self, to: u64) -> BinaryResult<u64> {
        Ok(self.0.seek(SeekFrom::Start(to))?)
    }

    fn tell(&mut self) -> BinaryResult<u64> {
        Ok(self.0.seek(SeekFrom::Current(0))?)
    }

    fn len(&self) -> BinaryResult<u64> {
        Ok(self.0.metadata()?.len())
    }
}

impl Read for FileStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.tell().unwrap() as usize + buffer.len()
            > self.0.metadata()?.len() as usize
        {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                BinaryError::ReadPastEof,
            ));
        }
        self.0.read(buffer)
    }
}

impl Write for FileStream {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl ReadStream for FileStream {}
impl WriteStream for FileStream {}

impl From<FileStream> for std::fs::File {
    fn from(value: FileStream) -> Self {
        value.0
    }
}
