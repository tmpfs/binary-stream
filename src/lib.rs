#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
//! Read and write binary data to streams.
//!
//! An asynchronous version using [futures::io](https://docs.rs/futures/latest/futures/io/index.html) is available using the `async` feature.
//!
//! Strings are length prefixed using `u32` by default, use
//! the `64bit` feature if you really need huge strings.
#![deny(missing_docs)]
use std::{
    borrow::Borrow,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write},
};

#[cfg(feature = "async")]
pub mod futures;

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

macro_rules! guard_size {
    ($len:expr, $max:expr) => {
        if let Some(max) = $max {
            if $len as usize > max {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "length {} exceeds max buffer size {}",
                        $len, max
                    ),
                ));
            }
        }
    };
}

pub(crate) use decode_endian;
pub(crate) use guard_size;

/// Variants to describe endianness.
#[derive(Clone, Copy)]
pub enum Endian {
    /// Big endian.
    Big,
    /// Little endian.
    Little,
}

impl Default for Endian {
    fn default() -> Self {
        Self::Little
    }
}

/// Options for reading and writing.
#[derive(Clone, Default)]
pub struct Options {
    /// The endian type.
    pub endian: Endian,
    /// Maximum buffer size for strings and byte slices.
    pub max_buffer_size: Option<usize>,
}

impl From<Endian> for Options {
    fn from(endian: Endian) -> Self {
        Self {
            endian,
            max_buffer_size: None,
        }
    }
}

/// Read from a stream.
pub struct BinaryReader<R>
where
    R: Read + Seek,
{
    stream: R,
    options: Options,
}

impl<R: Read + Seek> BinaryReader<R> {
    /// Create a binary reader with the given options.
    pub fn new(stream: R, options: Options) -> Self {
        Self { stream, options }
    }

    /// Seek to a position.
    pub fn seek(&mut self, to: SeekFrom) -> Result<u64> {
        Ok(self.stream.seek(to)?)
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
        self.seek(SeekFrom::Start(position))?;
        Ok(length)
    }

    /// Read a length-prefixed `String` from the stream.
    pub fn read_string(&mut self) -> Result<String> {
        let chars = if cfg!(feature = "64bit") {
            let str_len = self.read_u64()?;
            guard_size!(str_len, self.options.max_buffer_size);
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars)?;
            chars
        } else {
            let str_len = self.read_u32()?;
            guard_size!(str_len, self.options.max_buffer_size);
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
        decode_endian!(self.options.endian, buffer, f32);
    }

    /// Read a `f64` from the stream.
    pub fn read_f64(&mut self) -> Result<f64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, f64);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub fn read_isize(&mut self) -> Result<isize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, isize);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub fn read_isize(&mut self) -> Result<isize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, isize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub fn read_usize(&mut self) -> Result<usize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, usize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub fn read_usize(&mut self) -> Result<usize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, usize);
    }

    /// Read a `u64` from the stream.
    pub fn read_u64(&mut self) -> Result<u64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, u64);
    }

    /// Read an `i64` from the stream.
    pub fn read_i64(&mut self) -> Result<i64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, i64);
    }

    /// Read a `u128` from the stream.
    pub fn read_u128(&mut self) -> Result<u128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, u128);
    }

    /// Read an `i128` from the stream.
    pub fn read_i128(&mut self) -> Result<i128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, i128);
    }

    /// Read a `u32` from the stream.
    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, u32);
    }

    /// Read an `i32` from the stream.
    pub fn read_i32(&mut self) -> Result<i32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, i32);
    }

    /// Read a `u16` from the stream.
    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, u16);
    }

    /// Read an `i16` from the stream.
    pub fn read_i16(&mut self) -> Result<i16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, i16);
    }

    /// Read a `u8` from the stream.
    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, u8);
    }

    /// Read an `i8` from the stream.
    pub fn read_i8(&mut self) -> Result<i8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer)?;
        decode_endian!(self.options.endian, buffer, i8);
    }

    /// Read bytes from the stream into a buffer.
    pub fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        guard_size!(length, self.options.max_buffer_size);
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
    options: Options,
}

impl<W: Write + Seek> BinaryWriter<W> {
    /// Create a binary writer with the given options.
    pub fn new(stream: W, options: Options) -> Self {
        Self { stream, options }
    }

    /// Seek to a position.
    pub fn seek(&mut self, to: SeekFrom) -> Result<u64> {
        Ok(self.stream.seek(to)?)
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
        self.seek(SeekFrom::Start(position))?;
        Ok(length)
    }

    /// Write a length-prefixed `String` to the stream.
    pub fn write_string<S: AsRef<str>>(&mut self, value: S) -> Result<usize> {
        let bytes = value.as_ref().as_bytes();
        guard_size!(bytes.len(), self.options.max_buffer_size);
        if cfg!(feature = "64bit") {
            self.write_u64(bytes.len() as u64)?;
        } else {
            self.write_u32(bytes.len() as u32)?;
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
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `f64` to the stream.
    pub fn write_f64<V: Borrow<f64>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `isize` to the stream.
    pub fn write_isize<V: Borrow<isize>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `usize` to the stream.
    pub fn write_usize<V: Borrow<usize>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u64` to the stream.
    pub fn write_u64<V: Borrow<u64>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i64` to the stream.
    pub fn write_i64<V: Borrow<i64>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u128` to the stream.
    pub fn write_u128<V: Borrow<u128>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i128` to the stream.
    pub fn write_i128<V: Borrow<i128>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u32` to the stream.
    pub fn write_u32<V: Borrow<u32>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i32` to the stream.
    pub fn write_i32<V: Borrow<i32>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u16` to the stream.
    pub fn write_u16<V: Borrow<u16>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i16` to the stream.
    pub fn write_i16<V: Borrow<i16>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u8` to the stream.
    pub fn write_u8<V: Borrow<u8>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i8` to the stream.
    pub fn write_i8<V: Borrow<i8>>(&mut self, value: V) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a byte buffer to the stream.
    pub fn write_bytes<B: AsRef<[u8]>>(&mut self, data: B) -> Result<usize> {
        guard_size!(data.as_ref().len(), self.options.max_buffer_size);
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

#[cfg(test)]
mod tests {
    use super::{BinaryReader, BinaryWriter, Endian, Options};
    use anyhow::Result;
    use std::io::{Cursor, SeekFrom};
    use tempfile::tempfile;

    #[test]
    fn max_buffer_size() -> Result<()> {
        let options = Options {
            endian: Endian::Little,
            max_buffer_size: Some(1024),
        };

        let mut buffer = Vec::new();
        let stream = Cursor::new(&mut buffer);
        let mut writer = BinaryWriter::new(stream, options.clone());

        let large_string = ".".repeat(2048);
        let result = writer.write_string(&large_string);
        assert!(result.is_err());

        let large_buffer = [0u8; 2048];
        let result = writer.write_bytes(&large_buffer);
        assert!(result.is_err());

        // Create invalid values for the read assertions
        let mut read_string_buffer = {
            let mut buffer = Vec::new();
            let stream = Cursor::new(&mut buffer);
            let mut writer = BinaryWriter::new(stream, Default::default());
            writer.write_string(&large_string)?;
            buffer
        };

        let mut read_bytes_buffer = {
            let mut buffer = Vec::new();
            let stream = Cursor::new(&mut buffer);
            let mut writer = BinaryWriter::new(stream, Default::default());
            writer.write_bytes(&large_buffer)?;
            buffer
        };

        let mut stream = Cursor::new(&mut read_string_buffer);
        let mut reader = BinaryReader::new(&mut stream, options.clone());
        let result = reader.read_string();
        assert!(result.is_err());

        let mut stream = Cursor::new(&mut read_bytes_buffer);
        let mut reader = BinaryReader::new(&mut stream, options.clone());
        let result = reader.read_bytes(2048);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn borrow_test() -> Result<()> {
        let mut buffer = Vec::new();
        let stream = Cursor::new(&mut buffer);

        let mut writer = BinaryWriter::new(stream, Default::default());
        writer.write_u8(8)?;
        writer.write_u8(&8)?;
        writer.write_i8(-8)?;
        writer.write_i8(&-8)?;

        writer.write_u16(16)?;
        writer.write_u16(&16)?;
        writer.write_i16(-16)?;
        writer.write_i16(&-16)?;

        writer.write_u32(32)?;
        writer.write_u32(&32)?;
        writer.write_i32(-32)?;
        writer.write_i32(&-32)?;

        writer.write_u64(64)?;
        writer.write_u64(&64)?;
        writer.write_i64(-64)?;
        writer.write_i64(&-64)?;

        writer.write_u128(128)?;
        writer.write_u128(&128)?;
        writer.write_i128(-128)?;
        writer.write_i128(&-128)?;

        writer.write_usize(64)?;
        writer.write_usize(&64)?;
        writer.write_isize(-64)?;
        writer.write_isize(&-64)?;

        writer.write_char('c')?;
        writer.write_char(&'c')?;

        writer.write_bool(true)?;
        writer.write_bool(&true)?;

        writer.write_string("foo")?;
        writer.write_string(String::from("foo"))?;

        let buf: Vec<u8> = vec![1, 2, 3, 4];
        let exp: Vec<u8> = buf.clone(); // for assertion

        writer.write_bytes(&buf)?;
        writer.write_bytes(buf)?;

        let mut stream = Cursor::new(&mut buffer);
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let value = (reader.read_u8()?, reader.read_u8()?);
        assert_eq!((8, 8), value);
        let value = (reader.read_i8()?, reader.read_i8()?);
        assert_eq!((-8, -8), value);

        let value = (reader.read_u16()?, reader.read_u16()?);
        assert_eq!((16, 16), value);
        let value = (reader.read_i16()?, reader.read_i16()?);
        assert_eq!((-16, -16), value);

        let value = (reader.read_u32()?, reader.read_u32()?);
        assert_eq!((32, 32), value);
        let value = (reader.read_i32()?, reader.read_i32()?);
        assert_eq!((-32, -32), value);

        let value = (reader.read_u64()?, reader.read_u64()?);
        assert_eq!((64, 64), value);
        let value = (reader.read_i64()?, reader.read_i64()?);
        assert_eq!((-64, -64), value);

        let value = (reader.read_u128()?, reader.read_u128()?);
        assert_eq!((128, 128), value);
        let value = (reader.read_i128()?, reader.read_i128()?);
        assert_eq!((-128, -128), value);

        let value = (reader.read_usize()?, reader.read_usize()?);
        assert_eq!((64, 64), value);
        let value = (reader.read_isize()?, reader.read_isize()?);
        assert_eq!((-64, -64), value);

        let value = (reader.read_char()?, reader.read_char()?);
        assert_eq!(('c', 'c'), value);

        let value = (reader.read_bool()?, reader.read_bool()?);
        assert_eq!((true, true), value);

        let value = (reader.read_string()?, reader.read_string()?);
        assert_eq!((String::from("foo"), String::from("foo")), value);

        let value = (reader.read_bytes(4)?, reader.read_bytes(4)?);
        assert_eq!((exp.clone(), exp), value);

        Ok(())
    }

    #[test]
    fn slice_test() -> Result<()> {
        let mut buffer = Vec::new();
        let mut stream = Cursor::new(&mut buffer);

        let mut writer = BinaryWriter::new(&mut stream, Default::default());
        writer.write_u32(42)?;
        writer.write_string("foo")?;
        writer.write_char('b')?;

        let mut buffer = stream.into_inner();

        if cfg!(feature = "64bit") {
            assert_eq!(19, buffer.len());
        } else {
            assert_eq!(15, buffer.len());
        }

        let mut stream = Cursor::new(&mut buffer);
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        reader.seek(SeekFrom::Start(0))?;
        let value = reader.read_u32()?;
        assert_eq!(42, value);

        assert_eq!(4, reader.tell()?);

        let value = reader.read_string()?;
        assert_eq!("foo", &value);

        let value = reader.read_char()?;
        assert_eq!('b', value);

        if cfg!(feature = "64bit") {
            assert_eq!(19, buffer.len());
        } else {
            assert_eq!(15, buffer.len());
        }

        Ok(())
    }

    #[test]
    fn seek_test() -> Result<()> {
        let temp: f32 = 50.0;
        let seek_loc = 5u64;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_bytes([16; 32].to_vec())?;
        writer.seek(SeekFrom::Start(seek_loc))?;
        assert_eq!(writer.tell()?, seek_loc);
        writer.write_f32(temp)?;

        let mut reader = BinaryReader::new(&mut file, Default::default());
        reader.seek(SeekFrom::Start(seek_loc))?;
        assert_eq!(reader.tell()?, seek_loc);
        let read_temp = reader.read_f32()?;

        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_f64() -> Result<()> {
        let temp: f64 = f64::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());
        writer.write_f64(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_f64()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_f32() -> Result<()> {
        let temp: f32 = f32::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_f32(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_f32()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_isize() -> Result<()> {
        let temp: isize = isize::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_isize(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_isize()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_usize() -> Result<()> {
        let temp: usize = usize::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_usize(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_usize()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_i64() -> Result<()> {
        let temp: i64 = i64::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_i64(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_i64()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_i128() -> Result<()> {
        let temp: i128 = i128::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_i128(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_i128()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_i32() -> Result<()> {
        let temp: i32 = i32::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_i32(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_i32()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_i16() -> Result<()> {
        let temp: i16 = i16::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_i16(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_i16()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_i8() -> Result<()> {
        let temp: i8 = i8::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_i8(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_i8()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_u64() -> Result<()> {
        let temp: u64 = u64::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_u64(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_u64()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_u128() -> Result<()> {
        let temp: u128 = u128::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_u128(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_u128()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_u32() -> Result<()> {
        let temp: u32 = u32::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_u32(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_u32()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_u16() -> Result<()> {
        let temp: u16 = u16::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_u16(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_u16()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_test_u8() -> Result<()> {
        let temp: u8 = u8::MAX;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_u8(temp)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_u8()?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_write_bytes() -> Result<()> {
        let count = 20;

        let temp = vec![16; count];

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_bytes(temp.clone())?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_temp = reader.read_bytes(count)?;
        assert_eq!(temp, read_temp);

        Ok(())
    }

    #[test]
    fn read_out_of_range() -> Result<()> {
        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());
        writer.write_f32(5.0)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());
        reader.read_f32()?;

        assert!(reader.read_f32().is_err());

        Ok(())
    }

    #[test]
    fn read_write_string() -> Result<()> {
        let temp = "Hello World";

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());
        writer.write_string(temp.to_string())?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());
        let string = reader.read_string()?;
        assert_eq!(temp, string);

        Ok(())
    }

    #[test]
    fn read_write_test_bool() -> Result<()> {
        let positive = true;
        let negative = false;

        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());

        writer.write_bool(positive)?;
        writer.write_bool(negative)?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let read_positive = reader.read_bool()?;
        let read_negative = reader.read_bool()?;
        assert_eq!(positive, read_positive);
        assert_eq!(negative, read_negative);

        Ok(())
    }

    #[test]
    fn read_write_from_memorystream() -> Result<()> {
        let value_a = 3.0;
        let value_b = 5.0;
        let mut buffer = Vec::new();
        let mut stream = Cursor::new(&mut buffer);

        let mut writer = BinaryWriter::new(&mut stream, Default::default());
        writer.write_f32(value_a)?;
        writer.write_f32(value_b)?;

        let mut reader = BinaryReader::new(&mut stream, Default::default());
        reader.seek(SeekFrom::Start(0))?;
        let value = reader.read_f32()?;
        assert_eq!(value_a, value);
        let value = reader.read_f32()?;
        assert_eq!(value_b, value);

        Ok(())
    }

    #[test]
    fn write_to_memorystream_overlapping() -> Result<()> {
        let mut buffer = Vec::new();
        let mut stream = Cursor::new(&mut buffer);

        let mut writer = BinaryWriter::new(&mut stream, Default::default());
        writer.write_f32(1.0)?;
        writer.write_f32(2.0)?;
        writer.write_f32(3.0)?;

        writer.seek(SeekFrom::Start(0))?;
        writer.write_f32(4.0)?;
        writer.write_f32(5.0)?;
        writer.write_f32(6.0)?;

        let mut reader = BinaryReader::new(&mut stream, Default::default());
        reader.seek(SeekFrom::Start(0))?;
        let value = reader.read_f32()?;
        assert_eq!(4.0, value);
        let value = reader.read_f32()?;
        assert_eq!(5.0, value);
        let value = reader.read_f32()?;
        assert_eq!(6.0, value);

        Ok(())
    }

    /*
    #[test]
    fn write_to_memorystream_into_vec() -> Result<()> {
        let mut buffer = Vec::new();
        let mut stream = Cursor::new(&mut buffer);
        let mut writer = BinaryWriter::new(&mut stream, Default::default());
        writer.write_f32(1.0)?;
        assert_eq!(4, buffer.len());
        Ok(())
    }
    */

    #[test]
    fn write_to_filestream_overlapping() -> Result<()> {
        let mut file = tempfile()?;
        let mut writer = BinaryWriter::new(&mut file, Default::default());
        writer.write_f32(1.0)?;
        writer.write_f32(2.0)?;
        writer.write_f32(3.0)?;

        writer.seek(SeekFrom::Start(0))?;
        writer.write_f32(4.0)?;
        writer.write_f32(5.0)?;
        writer.write_f32(6.0)?;

        //let file = std::fs::File::open("filestream_overlapping.test")?;

        writer.seek(SeekFrom::Start(0))?;
        let mut reader = BinaryReader::new(&mut file, Default::default());
        let value = reader.read_f32()?;
        assert_eq!(4.0, value);
        let value = reader.read_f32()?;
        assert_eq!(5.0, value);
        let value = reader.read_f32()?;
        assert_eq!(6.0, value);

        assert_eq!(12, file.metadata()?.len());

        Ok(())
    }
}
