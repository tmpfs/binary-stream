//! Stream for operating on files.
use crate::{BinaryError, BinaryResult, ReadStream, SeekStream, WriteStream};
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Read, SeekFrom, Write};

/// Stream that wraps a file.
pub struct FileStream(pub std::fs::File);

impl SeekStream for FileStream {
    fn seek(&mut self, to: usize) -> BinaryResult<usize> {
        Ok(self.0.seek(SeekFrom::Start(to as u64))? as usize)
    }

    fn tell(&mut self) -> BinaryResult<usize> {
        Ok(self.0.seek(SeekFrom::Current(0))? as usize)
    }

    fn len(&self) -> BinaryResult<usize> {
        Ok(self.0.metadata()?.len().try_into()?)
    }
}

impl Read for FileStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.tell().unwrap() + buffer.len() > self.0.metadata()?.len() as usize {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                BinaryError::ReadPastEof,
            ));
        }
        Ok(self.0.read(buffer)?)
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
