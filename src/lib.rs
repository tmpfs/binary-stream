#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
//! Library for reading and writing binary data.
//!
//! An asynchronous version for `tokio` is available using
//! the `tokio` feature.
//!
//! Strings are length prefixed using `u64` by default, use
//! the `32bit` feature to use `u32` for the string length prefix.
#![deny(missing_docs)]
use std::{
    borrow::Borrow,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write},
};

#[cfg(feature = "tokio")]
pub mod tokio;

macro_rules! encode_endian {
    ($endian:expr, $value:expr, $stream:expr) => {
        let data = match $endian {
            Endian::Little => $value.to_le_bytes(),
            Endian::Big => $value.to_be_bytes(),
        };
        return Ok($stream.write(&data)?);
    };
}

macro_rules! decode_endian {
    ($endian:expr, $value:expr, $kind:ty) => {
        let data = match $endian {
            Endian::Little => <$kind>::from_le_bytes($value),
            Endian::Big => <$kind>::from_be_bytes($value),
        };
        return Ok(data);
    };
}

//pub(crate) use encode_endian;
pub(crate) use decode_endian;

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

/// Read from a stream.
pub struct BinaryReader<R>
where
    R: Read + Seek,
{
    stream: R,
    endian: Endian,
}

impl<R: Read + Seek> BinaryReader<R> {
    /// Create a binary reader with the given endianness.
    pub fn new(stream: R, endian: Endian) -> Self {
        Self { stream, endian }
    }

    /// Seek to a position.
    pub fn seek(&mut self, to: u64) -> Result<u64> {
        Ok(self.stream.seek(SeekFrom::Start(to))?)
    }

    /// Get the current seek position.
    pub fn tell(&mut self) -> Result<u64> {
        Ok(self.stream.stream_position()?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub fn len(&mut self) -> Result<u64> {
        let position = self.tell()?;
        let length = self.stream.seek(SeekFrom::End(0))?;
        self.seek(position)?;
        Ok(length)
    }

    /// Read a length-prefixed `String` from the stream.
    pub fn read_string(&mut self) -> Result<String> {
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
        Ok(String::from_utf8(chars)
            .map_err(|_| Error::new(ErrorKind::Other, "invalid utf-8"))?)
    }

    /// Read a character from the stream.
    pub fn read_char(&mut self) -> Result<char> {
        std::char::from_u32(self.read_u32()?)
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid character"))
    }

    /// Read a `bool` from the stream.
    pub fn read_bool(&mut self) -> Result<bool> {
        let value = self.read_u8()?;
        Ok(value > 0)
    }

    /// Read a `f32` from the stream.
    pub fn read_f32(&mut self) -> Result<f32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, f32);
    }

    /// Read a `f64` from the stream.
    pub fn read_f64(&mut self) -> Result<f64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, f64);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub fn read_isize(&mut self) -> Result<isize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, isize);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub fn read_isize(&mut self) -> Result<isize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, isize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub fn read_usize(&mut self) -> Result<usize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, usize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub fn read_usize(&mut self) -> Result<usize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, usize);
    }

    /// Read a `u64` from the stream.
    pub fn read_u64(&mut self) -> Result<u64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, u64);
    }

    /// Read an `i64` from the stream.
    pub fn read_i64(&mut self) -> Result<i64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, i64);
    }

    /// Read a `u128` from the stream.
    pub fn read_u128(&mut self) -> Result<u128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, u128);
    }

    /// Read an `i128` from the stream.
    pub fn read_i128(&mut self) -> Result<i128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, i128);
    }

    /// Read a `u32` from the stream.
    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, u32);
    }

    /// Read an `i32` from the stream.
    pub fn read_i32(&mut self) -> Result<i32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, i32);
    }

    /// Read a `u16` from the stream.
    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, u16);
    }

    /// Read an `i16` from the stream.
    pub fn read_i16(&mut self) -> Result<i16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, i16);
    }

    /// Read a `u8` from the stream.
    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, u8);
    }

    /// Read an `i8` from the stream.
    pub fn read_i8(&mut self) -> Result<i8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.endian, buffer, i8);
    }

    /// Read bytes from the stream into a buffer.
    pub fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![0; length];
        self.stream.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

/// Write to a stream.
pub struct BinaryWriter<W>
where
    W: Write + Seek,
{
    stream: W,
    endian: Endian,
}

impl<W: Write + Seek> BinaryWriter<W> {
    /// Create a binary writer with the given endianness.
    pub fn new(stream: W, endian: Endian) -> Self {
        Self { stream, endian }
    }

    /// Seek to a position.
    pub fn seek(&mut self, to: u64) -> Result<u64> {
        Ok(self.stream.seek(SeekFrom::Start(to))?)
    }

    /// Get the current seek position.
    pub fn tell(&mut self) -> Result<u64> {
        Ok(self.stream.stream_position()?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub fn len(&mut self) -> Result<u64> {
        let position = self.tell()?;
        let length = self.stream.seek(SeekFrom::End(0))?;
        self.seek(position)?;
        Ok(length)
    }

    /// Write a length-prefixed `String` to the stream.
    pub fn write_string<S: AsRef<str>>(&mut self, value: S) -> Result<usize> {
        let bytes = value.as_ref().as_bytes();
        if cfg!(feature = "32bit") {
            self.write_u32(bytes.len() as u32)?;
        } else {
            self.write_u64(bytes.len() as u64)?;
        }
        Ok(self.stream.write(bytes)?)
    }

    /// Write a character to the stream.
    pub fn write_char<V: Borrow<char>>(&mut self, v: V) -> Result<usize> {
        self.write_u32(*v.borrow() as u32)
    }

    /// Write a `bool` to the stream.
    pub fn write_bool<V: Borrow<bool>>(&mut self, value: V) -> Result<usize> {
        let written = self.write_u8(if *value.borrow() { 1 } else { 0 })?;
        Ok(written)
    }

    /// Write a `f32` to the stream.
    pub fn write_f32<V: Borrow<f32>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `f64` to the stream.
    pub fn write_f64<V: Borrow<f64>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `isize` to the stream.
    pub fn write_isize<V: Borrow<isize>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `usize` to the stream.
    pub fn write_usize<V: Borrow<usize>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u64` to the stream.
    pub fn write_u64<V: Borrow<u64>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i64` to the stream.
    pub fn write_i64<V: Borrow<i64>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u128` to the stream.
    pub fn write_u128<V: Borrow<u128>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i128` to the stream.
    pub fn write_i128<V: Borrow<i128>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u32` to the stream.
    pub fn write_u32<V: Borrow<u32>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i32` to the stream.
    pub fn write_i32<V: Borrow<i32>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u16` to the stream.
    pub fn write_u16<V: Borrow<u16>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i16` to the stream.
    pub fn write_i16<V: Borrow<i16>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u8` to the stream.
    pub fn write_u8<V: Borrow<u8>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i8` to the stream.
    pub fn write_i8<V: Borrow<i8>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a byte buffer to the stream.
    pub fn write_bytes<B: AsRef<[u8]>>(&mut self, data: B) -> Result<usize> {
        Ok(self.stream.write(data.as_ref())?)
    }
}

/// Trait for encoding to binary.
pub trait Encode {
    /// Encode self into the binary writer.
    fn encode<W: Write + Seek>(
        &self,
        writer: &mut BinaryWriter<W>,
    ) -> Result<()>;
}

/// Trait for decoding from binary.
pub trait Decode {
    /// Decode from the binary reader into self.
    fn decode<R: Read + Seek>(
        &mut self,
        reader: &mut BinaryReader<R>,
    ) -> Result<()>;
}
