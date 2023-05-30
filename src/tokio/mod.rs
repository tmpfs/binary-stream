//! Asynchronous reader and writer for tokio.
use std::{borrow::Borrow, io::SeekFrom};
use tokio::io::{AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWriteExt};
use async_trait::async_trait;

use crate::{decode_endian, BinaryError, BinaryResult, Endian};

macro_rules! encode_endian {
    ($endian:expr, $value:expr, $stream:expr) => {
        let data = match $endian {
            Endian::Little => $value.to_le_bytes(),
            Endian::Big => $value.to_be_bytes(),
        };
        return Ok($stream.write(&data).await?);
    };
}

/// Read from a stream.
pub struct BinaryReader<R>
where
    R: AsyncReadExt + AsyncSeek + Unpin,
{
    stream: R,
    endian: Endian,
}

impl<R: AsyncReadExt + AsyncSeek + Unpin> BinaryReader<R> {
    /// Create a binary reader with the given endianness.
    pub fn new(stream: R, endian: Endian) -> Self {
        Self { stream, endian }
    }

    /// Seek to a position.
    pub async fn seek(&mut self, to: u64) -> BinaryResult<u64> {
        Ok(self.stream.seek(SeekFrom::Start(to)).await?)
    }

    /// Get the current position.
    pub async fn tell(&mut self) -> BinaryResult<u64> {
        Ok(self.stream.stream_position().await?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub async fn len(&mut self) -> BinaryResult<u64> {
        let position = self.tell().await?;
        let length = self.stream.seek(SeekFrom::End(0)).await?;
        self.seek(position).await?;
        Ok(length)
    }

    /// Read a length-prefixed `String` from the stream.
    pub async fn read_string(&mut self) -> BinaryResult<String> {
        let chars = if cfg!(feature = "32bit") {
            let str_len = self.read_u32().await?;
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars).await?;
            chars
        } else {
            let str_len = self.read_u64().await?;
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars).await?;
            chars
        };
        Ok(String::from_utf8(chars)?)
    }

    /// Read a character from the stream.
    pub async fn read_char(&mut self) -> BinaryResult<char> {
        std::char::from_u32(self.read_u32().await?)
            .ok_or(BinaryError::InvalidChar)
    }

    /// Read a `bool` from the stream.
    pub async fn read_bool(&mut self) -> BinaryResult<bool> {
        let value = self.read_u8().await?;
        Ok(value > 0)
    }

    /// Read a `f32` from the stream.
    pub async fn read_f32(&mut self) -> BinaryResult<f32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, f32);
    }

    /// Read a `f64` from the stream.
    pub async fn read_f64(&mut self) -> BinaryResult<f64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, f64);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub async fn read_isize(&mut self) -> BinaryResult<isize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, isize);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub async fn read_isize(&mut self) -> BinaryResult<isize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, isize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub async fn read_usize(&mut self) -> BinaryResult<usize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, usize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub async fn read_usize(&mut self) -> BinaryResult<usize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, usize);
    }

    /// Read a `u64` from the stream.
    pub async fn read_u64(&mut self) -> BinaryResult<u64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, u64);
    }

    /// Read an `i64` from the stream.
    pub async fn read_i64(&mut self) -> BinaryResult<i64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, i64);
    }

    /// Read a `u128` from the stream.
    pub async fn read_u128(&mut self) -> BinaryResult<u128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, u128);
    }

    /// Read an `i128` from the stream.
    pub async fn read_i128(&mut self) -> BinaryResult<i128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, i128);
    }

    /// Read a `u32` from the stream.
    pub async fn read_u32(&mut self) -> BinaryResult<u32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, u32);
    }

    /// Read an `i32` from the stream.
    pub async fn read_i32(&mut self) -> BinaryResult<i32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, i32);
    }

    /// Read a `u16` from the stream.
    pub async fn read_u16(&mut self) -> BinaryResult<u16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, u16);
    }

    /// Read an `i16` from the stream.
    pub async fn read_i16(&mut self) -> BinaryResult<i16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, i16);
    }

    /// Read a `u8` from the stream.
    pub async fn read_u8(&mut self) -> BinaryResult<u8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, u8);
    }

    /// Read an `i8` from the stream.
    pub async fn read_i8(&mut self) -> BinaryResult<i8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.endian, buffer, i8);
    }

    /// Read bytes from the stream into a buffer.
    pub async fn read_bytes(
        &mut self,
        length: usize,
    ) -> BinaryResult<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![0; length];
        self.stream.read_exact(&mut buffer).await?;
        Ok(buffer)
    }
}

/// Write to a stream.
pub struct BinaryWriter<W>
where
    W: AsyncWriteExt + AsyncSeek + Unpin,
{
    stream: W,
    endian: Endian,
}

impl<W: AsyncWriteExt + AsyncSeek + Unpin> BinaryWriter<W> {
    /// Create a binary writer with the given endianness.
    pub fn new(stream: W, endian: Endian) -> Self {
        Self { stream, endian }
    }

    /// Seek to a position.
    pub async fn seek(&mut self, to: u64) -> BinaryResult<u64> {
        Ok(self.stream.seek(SeekFrom::Start(to)).await?)
    }

    /// Get the current position.
    pub async fn tell(&mut self) -> BinaryResult<u64> {
        Ok(self.stream.stream_position().await?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub async fn len(&mut self) -> BinaryResult<u64> {
        let position = self.tell().await?;
        let length = self.stream.seek(SeekFrom::End(0)).await?;
        self.seek(position).await?;
        Ok(length)
    }

    /// Write a length-prefixed `String` to the stream.
    pub async fn write_string<S: AsRef<str>>(
        &mut self,
        value: S,
    ) -> BinaryResult<usize> {
        let bytes = value.as_ref().as_bytes();
        if cfg!(feature = "32bit") {
            self.write_u32(bytes.len() as u32).await?;
        } else {
            self.write_u64(bytes.len() as u64).await?;
        }
        Ok(self.stream.write(bytes).await?)
    }

    /// Write a character to the stream.
    pub async fn write_char<V: Borrow<char>>(
        &mut self,
        v: V,
    ) -> BinaryResult<usize> {
        self.write_u32(*v.borrow() as u32).await
    }

    /// Write a `bool` to the stream.
    pub async fn write_bool<V: Borrow<bool>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        let written =
            self.write_u8(if *value.borrow() { 1 } else { 0 }).await?;
        Ok(written)
    }

    /// Write a `f32` to the stream.
    pub async fn write_f32<V: Borrow<f32>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `f64` to the stream.
    pub async fn write_f64<V: Borrow<f64>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `isize` to the stream.
    pub async fn write_isize<V: Borrow<isize>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `usize` to the stream.
    pub async fn write_usize<V: Borrow<usize>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u64` to the stream.
    pub async fn write_u64<V: Borrow<u64>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i64` to the stream.
    pub async fn write_i64<V: Borrow<i64>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u128` to the stream.
    pub async fn write_u128<V: Borrow<u128>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i128` to the stream.
    pub async fn write_i128<V: Borrow<i128>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u32` to the stream.
    pub async fn write_u32<V: Borrow<u32>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i32` to the stream.
    pub async fn write_i32<V: Borrow<i32>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u16` to the stream.
    pub async fn write_u16<V: Borrow<u16>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i16` to the stream.
    pub async fn write_i16<V: Borrow<i16>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a `u8` to the stream.
    pub async fn write_u8<V: Borrow<u8>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write an `i8` to the stream.
    pub async fn write_i8<V: Borrow<i8>>(
        &mut self,
        value: V,
    ) -> BinaryResult<usize> {
        encode_endian!(self.endian, value.borrow(), self.stream);
    }

    /// Write a byte buffer to the stream.
    pub async fn write_bytes<B: AsRef<[u8]>>(
        &mut self,
        data: B,
    ) -> BinaryResult<usize> {
        Ok(self.stream.write(data.as_ref()).await?)
    }
}

#[cfg(test)]
mod test {
    use super::{BinaryReader, BinaryWriter};
    use crate::Endian;
    use anyhow::Result;

    #[tokio::test]
    async fn async_tokio() -> Result<()> {
        let mock_str = String::from("mock value");
        let mock_char = 'c';

        let mut write_file =
            tokio::fs::File::create("target/async-tokio.test").await?;
        let mut writer = BinaryWriter::new(&mut write_file, Endian::Little);
        writer.write_string(&mock_str).await?;
        writer.write_char(&mock_char).await?;

        let mut read_file =
            tokio::fs::File::open("target/async-tokio.test").await?;
        let mut reader = BinaryReader::new(&mut read_file, Endian::Little);

        let str_value = reader.read_string().await?;
        assert_eq!(mock_str, str_value);
        let char_value = reader.read_char().await?;
        assert_eq!(mock_char, char_value);

        Ok(())
    }
}


/// Trait for encoding to binary.
#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Encode {
    /// Encode self into the binary writer.
    async fn encode<W: AsyncWriteExt + AsyncSeek + Unpin>(
        &self,
        writer: &mut BinaryWriter<W>,
    ) -> BinaryResult<()>;
}

/// Trait for decoding from binary.
#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Decode {
    /// Decode from the binary reader into self.
    async fn decode<R: AsyncReadExt + AsyncSeek + Unpin>(
        &mut self,
        reader: &mut BinaryReader<R>,
    ) -> BinaryResult<()>;
}
