//! Library for reading and writing binary data.
//!
//! Strings are length prefixed using `u64` by default, use 
//! the `32bit` feature to use `u32` for the string length prefix.
#![deny(missing_docs)]
use std::{
    borrow::Borrow,
    io::{Read, Write},
};

mod error;
mod stream;

pub use error::BinaryError;
pub use stream::file::FileStream;
pub use stream::memory::MemoryStream;
pub use stream::slice::SliceStream;

/// BinaryResult type for binary errors.
pub type BinaryResult<T> = std::result::Result<T, BinaryError>;

macro_rules! encode {
    ($endian:expr, $value:expr, $stream:expr) => {
        let data = match $endian {
            Endian::Little => $value.to_le_bytes(),
            Endian::Big => $value.to_be_bytes(),
        };
        return Ok($stream.write(&data)?);
    };
}

macro_rules! decode {
    ($endian:expr, $value:expr, $kind:ty) => {
        let data = match $endian {
            Endian::Little => <$kind>::from_le_bytes($value),
            Endian::Big => <$kind>::from_be_bytes($value),
        };
        return Ok(data);
    };
}

/// Variants to describe endianness.
pub enum Endian {
    /// Big endian.
    Big,
    /// Little endian.
    Little,
}

impl Default for Endian {
    fn default() -> Self {
        Self::Big
    }
}

/// Trait for streams that can seek.
#[allow(clippy::len_without_is_empty)]
pub trait SeekStream {
    /// Seek to a position.
    fn seek(&mut self, to: u64) -> BinaryResult<u64>;
    /// Get the current position.
    fn tell(&mut self) -> BinaryResult<u64>;
    /// Get the length of the stream.
    fn len(&self) -> BinaryResult<u64>;
}

/// Trait for a readable stream.
pub trait ReadStream: Read + SeekStream {}

/// Trait for a writable stream.
pub trait WriteStream: Write + SeekStream {}

/// Read from a stream.
pub struct BinaryReader<'a> {
    stream: &'a mut dyn ReadStream,
    endian: Endian,
}

impl<'a> SeekStream for BinaryReader<'a> {
    fn seek(&mut self, to: u64) -> BinaryResult<u64> {
        self.stream.seek(to)
    }

    fn tell(&mut self) -> BinaryResult<u64> {
        self.stream.tell()
    }

    fn len(&self) -> BinaryResult<u64> {
        self.stream.len()
    }
}

impl<'a> BinaryReader<'a> {
    /// Create a binary reader with the given endianness.
    pub fn new(stream: &'a mut dyn ReadStream, endian: Endian) -> Self {
        Self { stream, endian }
    }

    /// Read a length-prefixed `String` from the stream.
    pub fn read_string(&mut self) -> BinaryResult<String> {
        let chars = if cfg!(feature = "32bit") {
            let str_len = self.read_u32()?;
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars)?;
            chars
        } else {
            let str_len = self.read_u64()?;
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars)?;
            chars
        };
        Ok(String::from_utf8(chars)?)
    }

    /// Read a character from the stream.
    pub fn read_char(&mut self) -> BinaryResult<char> {
        std::char::from_u32(self.read_u32()?).ok_or(BinaryError::InvalidChar)
    }

    /// Read a `bool` from the stream.
    pub fn read_bool(&mut self) -> BinaryResult<bool> {
        let value = self.read_u8()?;
        Ok(value > 0)
    }

    /// Read a `f32` from the stream.
    pub fn read_f32(&mut self) -> BinaryResult<f32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, f32);
    }

    /// Read a `f64` from the stream.
    pub fn read_f64(&mut self) -> BinaryResult<f64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, f64);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub fn read_isize(&mut self) -> BinaryResult<isize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, isize);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub fn read_isize(&mut self) -> BinaryResult<isize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, isize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub fn read_usize(&mut self) -> BinaryResult<usize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, usize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub fn read_usize(&mut self) -> BinaryResult<usize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, usize);
    }

    /// Read a `u64` from the stream.
    pub fn read_u64(&mut self) -> BinaryResult<u64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, u64);
    }

    /// Read an `i64` from the stream.
    pub fn read_i64(&mut self) -> BinaryResult<i64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, i64);
    }

    /// Read a `u128` from the stream.
    pub fn read_u128(&mut self) -> BinaryResult<u128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, u128);
    }

    /// Read an `i128` from the stream.
    pub fn read_i128(&mut self) -> BinaryResult<i128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, i128);
    }

    /// Read a `u32` from the stream.
    pub fn read_u32(&mut self) -> BinaryResult<u32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, u32);
    }

    /// Read an `i32` from the stream.
    pub fn read_i32(&mut self) -> BinaryResult<i32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, i32);
    }

    /// Read a `u16` from the stream.
    pub fn read_u16(&mut self) -> BinaryResult<u16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, u16);
    }

    /// Read an `i16` from the stream.
    pub fn read_i16(&mut self) -> BinaryResult<i16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, i16);
    }

    /// Read a `u8` from the stream.
    pub fn read_u8(&mut self) -> BinaryResult<u8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, u8);
    }

    /// Read an `i8` from the stream.
    pub fn read_i8(&mut self) -> BinaryResult<i8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer)?;
        decode!(self.endian, buffer, i8);
    }

    /// Read bytes from the stream into a buffer.
    pub fn read_bytes(&mut self, length: usize) -> BinaryResult<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![0; length];
        self.stream.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

/// Write to a stream.
pub struct BinaryWriter<'a> {
    stream: &'a mut dyn WriteStream,
    endian: Endian,
}

impl<'a> SeekStream for BinaryWriter<'a> {
    fn seek(&mut self, to: u64) -> BinaryResult<u64> {
        self.stream.seek(to)
    }

    fn tell(&mut self) -> BinaryResult<u64> {
        self.stream.tell()
    }

    fn len(&self) -> BinaryResult<u64> {
        self.stream.len()
    }
}

impl<'a> BinaryWriter<'a> {
    /// Create a binary writer with the given endianness.
    pub fn new(stream: &'a mut dyn WriteStream, endian: Endian) -> Self {
        Self { stream, endian }
    }

    /// Write a length-prefixed `String` to the stream.
    pub fn write_string<S: AsRef<str>>(
        &mut self,
        value: S,
    ) -> BinaryResult<usize> {
        let bytes = value.as_ref().as_bytes();
        if cfg!(feature = "32bit") {
            self.write_u32(bytes.len() as u32)?;
        } else {
            self.write_u64(bytes.len() as u64)?;
        }
        Ok(self.stream.write(bytes)?)
    }

    /// Write a character to the stream.
    pub fn write_char<V: Borrow<char>>(
        &mut self,
        v: V,
    ) -> BinaryResult<usize> {
        self.write_u32(*v.borrow() as u32)
    }

    /// Write a `bool` to the stream.
    pub fn write_bool<V: Borrow<bool>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        let written = self.write_u8(if *value.borrow() { 1 } else { 0 })?;
        Ok(written)
    }

    /// Write a `f32` to the stream.
    pub fn write_f32<V: Borrow<f32>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `f64` to the stream.
    pub fn write_f64<V: Borrow<f64>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `isize` to the stream.
    pub fn write_isize<V: Borrow<isize>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `usize` to the stream.
    pub fn write_usize<V: Borrow<usize>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u64` to the stream.
    pub fn write_u64<V: Borrow<u64>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i64` to the stream.
    pub fn write_i64<V: Borrow<i64>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u128` to the stream.
    pub fn write_u128<V: Borrow<u128>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i128` to the stream.
    pub fn write_i128<V: Borrow<i128>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u32` to the stream.
    pub fn write_u32<V: Borrow<u32>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i32` to the stream.
    pub fn write_i32<V: Borrow<i32>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u16` to the stream.
    pub fn write_u16<V: Borrow<u16>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i16` to the stream.
    pub fn write_i16<V: Borrow<i16>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u8` to the stream.
    pub fn write_u8<V: Borrow<u8>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i8` to the stream.
    pub fn write_i8<V: Borrow<i8>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode!(self.endian, value.borrow(), self.stream);
    }

    /// Write a byte buffer to the stream.
    pub fn write_bytes<B: AsRef<[u8]>>(
        &mut self,
        data: B,
    ) -> BinaryResult<usize> {
        Ok(self.stream.write(data.as_ref())?)
    }
}

/// Trait for encoding to binary.
pub trait Encode {
    /// Encode self into the binary writer.
    fn encode(&self, writer: &mut BinaryWriter) -> BinaryResult<()>;
}

/// Trait for decoding from binary.
pub trait Decode {
    /// Decode from the binary reader into self.
    fn decode(&mut self, reader: &mut BinaryReader) -> BinaryResult<()>;
}
