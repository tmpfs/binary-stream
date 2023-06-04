//! Asynchronous reader and writer for tokio.
use async_trait::async_trait;
use std::{
    borrow::Borrow,
    io::{Error, ErrorKind, Result, SeekFrom},
};
use tokio::io::{
    AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite,
    AsyncWriteExt,
};

use crate::{decode_endian, Endian, Options};

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
    R: AsyncRead + AsyncSeek + Unpin,
{
    stream: R,
    options: Options,
}

impl<R: AsyncRead + AsyncSeek + Unpin> BinaryReader<R> {
    /// Create a binary reader with the given options.
    pub fn new(stream: R, options: Options) -> Self {
        Self { stream, options }
    }

    /// Seek to a position.
    pub async fn seek(&mut self, to: u64) -> Result<u64> {
        Ok(self.stream.seek(SeekFrom::Start(to)).await?)
    }

    /// Get the current position.
    pub async fn tell(&mut self) -> Result<u64> {
        Ok(self.stream.stream_position().await?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub async fn len(&mut self) -> Result<u64> {
        let position = self.tell().await?;
        let length = self.stream.seek(SeekFrom::End(0)).await?;
        self.seek(position).await?;
        Ok(length)
    }

    /// Read a length-prefixed `String` from the stream.
    pub async fn read_string(&mut self) -> Result<String> {
        let chars = if cfg!(feature = "32bit") {
            let str_len = self.read_u32().await?;
            guard_size!(str_len, self.options.max_buffer_size);
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars).await?;
            chars
        } else {
            let str_len = self.read_u64().await?;
            guard_size!(str_len, self.options.max_buffer_size);
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars).await?;
            chars
        };
        Ok(String::from_utf8(chars)
            .map_err(|_| Error::new(ErrorKind::Other, "invalid utf-8"))?)
    }

    /// Read a character from the stream.
    pub async fn read_char(&mut self) -> Result<char> {
        std::char::from_u32(self.read_u32().await?)
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid character"))
    }

    /// Read a `bool` from the stream.
    pub async fn read_bool(&mut self) -> Result<bool> {
        let value = self.read_u8().await?;
        Ok(value > 0)
    }

    /// Read a `f32` from the stream.
    pub async fn read_f32(&mut self) -> Result<f32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, f32);
    }

    /// Read a `f64` from the stream.
    pub async fn read_f64(&mut self) -> Result<f64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, f64);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub async fn read_isize(&mut self) -> Result<isize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, isize);
    }

    /// Read an `isize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub async fn read_isize(&mut self) -> Result<isize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, isize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "32")]
    pub async fn read_usize(&mut self) -> Result<usize> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, usize);
    }

    /// Read a `usize` from the stream.
    #[cfg(target_pointer_width = "64")]
    pub async fn read_usize(&mut self) -> Result<usize> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, usize);
    }

    /// Read a `u64` from the stream.
    pub async fn read_u64(&mut self) -> Result<u64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, u64);
    }

    /// Read an `i64` from the stream.
    pub async fn read_i64(&mut self) -> Result<i64> {
        let mut buffer: [u8; 8] = [0; 8];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, i64);
    }

    /// Read a `u128` from the stream.
    pub async fn read_u128(&mut self) -> Result<u128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, u128);
    }

    /// Read an `i128` from the stream.
    pub async fn read_i128(&mut self) -> Result<i128> {
        let mut buffer: [u8; 16] = [0; 16];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, i128);
    }

    /// Read a `u32` from the stream.
    pub async fn read_u32(&mut self) -> Result<u32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, u32);
    }

    /// Read an `i32` from the stream.
    pub async fn read_i32(&mut self) -> Result<i32> {
        let mut buffer: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, i32);
    }

    /// Read a `u16` from the stream.
    pub async fn read_u16(&mut self) -> Result<u16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, u16);
    }

    /// Read an `i16` from the stream.
    pub async fn read_i16(&mut self) -> Result<i16> {
        let mut buffer: [u8; 2] = [0; 2];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, i16);
    }

    /// Read a `u8` from the stream.
    pub async fn read_u8(&mut self) -> Result<u8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, u8);
    }

    /// Read an `i8` from the stream.
    pub async fn read_i8(&mut self) -> Result<i8> {
        let mut buffer: [u8; 1] = [0; 1];
        self.stream.read_exact(&mut buffer).await?;
        decode_endian!(self.options.endian, buffer, i8);
    }

    /// Read bytes from the stream into a buffer.
    pub async fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        guard_size!(length, self.options.max_buffer_size);
        let mut buffer: Vec<u8> = vec![0; length];
        self.stream.read_exact(&mut buffer).await?;
        Ok(buffer)
    }
}

/// Write to a stream.
pub struct BinaryWriter<W>
where
    W: AsyncWrite + AsyncSeek + Unpin,
{
    stream: W,
    options: Options,
}

impl<W: AsyncWrite + AsyncSeek + Unpin> BinaryWriter<W> {
    /// Create a binary writer with the given options.
    pub fn new(stream: W, options: Options) -> Self {
        Self { stream, options }
    }

    /// Seek to a position.
    pub async fn seek(&mut self, to: u64) -> Result<u64> {
        Ok(self.stream.seek(SeekFrom::Start(to)).await?)
    }

    /// Get the current position.
    pub async fn tell(&mut self) -> Result<u64> {
        Ok(self.stream.stream_position().await?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub async fn len(&mut self) -> Result<u64> {
        let position = self.tell().await?;
        let length = self.stream.seek(SeekFrom::End(0)).await?;
        self.seek(position).await?;
        Ok(length)
    }

    /// Write a length-prefixed `String` to the stream.
    pub async fn write_string<S: AsRef<str>>(
        &mut self,
        value: S,
    ) -> Result<usize> {
        let bytes = value.as_ref().as_bytes();
        guard_size!(bytes.len(), self.options.max_buffer_size);
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
    ) -> Result<usize> {
        self.write_u32(*v.borrow() as u32).await
    }

    /// Write a `bool` to the stream.
    pub async fn write_bool<V: Borrow<bool>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        let written =
            self.write_u8(if *value.borrow() { 1 } else { 0 }).await?;
        Ok(written)
    }

    /// Write a `f32` to the stream.
    pub async fn write_f32<V: Borrow<f32>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `f64` to the stream.
    pub async fn write_f64<V: Borrow<f64>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `isize` to the stream.
    pub async fn write_isize<V: Borrow<isize>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `usize` to the stream.
    pub async fn write_usize<V: Borrow<usize>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u64` to the stream.
    pub async fn write_u64<V: Borrow<u64>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i64` to the stream.
    pub async fn write_i64<V: Borrow<i64>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u128` to the stream.
    pub async fn write_u128<V: Borrow<u128>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i128` to the stream.
    pub async fn write_i128<V: Borrow<i128>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u32` to the stream.
    pub async fn write_u32<V: Borrow<u32>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i32` to the stream.
    pub async fn write_i32<V: Borrow<i32>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u16` to the stream.
    pub async fn write_u16<V: Borrow<u16>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i16` to the stream.
    pub async fn write_i16<V: Borrow<i16>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a `u8` to the stream.
    pub async fn write_u8<V: Borrow<u8>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write an `i8` to the stream.
    pub async fn write_i8<V: Borrow<i8>>(
        &mut self,
        value: V,
    ) -> Result<usize> {
        encode_endian!(self.options.endian, value.borrow(), self.stream);
    }

    /// Write a byte buffer to the stream.
    pub async fn write_bytes<B: AsRef<[u8]>>(
        &mut self,
        data: B,
    ) -> Result<usize> {
        guard_size!(data.as_ref().len(), self.options.max_buffer_size);
        Ok(self.stream.write(data.as_ref()).await?)
    }
}

/// Trait for encoding to binary.
#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Encode {
    /// Encode self into the binary writer.
    async fn encode<W: AsyncWrite + AsyncSeek + Unpin + Send>(
        &self,
        writer: &mut BinaryWriter<W>,
    ) -> Result<()>;
}

/// Trait for decoding from binary.
#[cfg_attr(target_arch="wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Decode {
    /// Decode from the binary reader into self.
    async fn decode<R: AsyncRead + AsyncSeek + Unpin + Send>(
        &mut self,
        reader: &mut BinaryReader<R>,
    ) -> Result<()>;
}

#[cfg(test)]
mod test {
    use super::{BinaryReader, BinaryWriter};
    use anyhow::Result;

    #[tokio::test]
    async fn async_tokio() -> Result<()> {
        let mock_str = String::from("mock value");
        let mock_char = 'c';

        let mut write_file =
            tokio::fs::File::create("target/async-tokio.test").await?;
        let mut writer =
            BinaryWriter::new(&mut write_file, Default::default());
        writer.write_string(&mock_str).await?;
        writer.write_char(&mock_char).await?;

        let mut read_file =
            tokio::fs::File::open("target/async-tokio.test").await?;
        let mut reader =
            BinaryReader::new(&mut read_file, Default::default());

        let str_value = reader.read_string().await?;
        assert_eq!(mock_str, str_value);
        let char_value = reader.read_char().await?;
        assert_eq!(mock_char, char_value);

        Ok(())
    }
}
