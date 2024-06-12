//! Asynchronous reader and writer for tokio.
use async_trait::async_trait;
use futures::io::{
    AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite,
    AsyncWriteExt, BufReader, BufWriter, Cursor,
};
use std::{
    borrow::Borrow,
    io::{Error, ErrorKind, Result, SeekFrom},
};

use crate::{decode_endian, guard_size, Endian, Options};

macro_rules! encode_endian {
    ($endian:expr, $value:expr, $stream:expr) => {
        let data = match $endian {
            Endian::Little => $value.to_le_bytes(),
            Endian::Big => $value.to_be_bytes(),
        };
        return Ok($stream.write(&data).await?);
    };
}

/// Get the length of a stream by seeking to the end
/// and then restoring the previous position.
pub async fn stream_length<S: AsyncSeek + Unpin>(
    stream: &mut S,
) -> Result<u64> {
    let position = stream.stream_position().await?;
    let length = stream.seek(SeekFrom::End(0)).await?;
    stream.seek(SeekFrom::Start(position)).await?;
    Ok(length)
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
    pub async fn seek(&mut self, to: SeekFrom) -> Result<u64> {
        Ok(self.stream.seek(to).await?)
    }

    /// Get the current position.
    pub async fn stream_position(&mut self) -> Result<u64> {
        Ok(self.stream.stream_position().await?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub async fn len(&mut self) -> Result<u64> {
        stream_length(&mut self.stream).await
    }

    /// Read a length-prefixed `String` from the stream.
    pub async fn read_string(&mut self) -> Result<String> {
        let chars = if cfg!(feature = "64bit") {
            let str_len = self.read_u64().await?;
            guard_size!(str_len, self.options.max_buffer_size);
            let mut chars: Vec<u8> = vec![0; str_len as usize];
            self.stream.read_exact(&mut chars).await?;
            chars
        } else {
            let str_len = self.read_u32().await?;
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
    pub async fn seek(&mut self, to: SeekFrom) -> Result<u64> {
        if cfg!(feature = "tokio-compat-flush") {
            self.stream.flush().await?;
        }
        Ok(self.stream.seek(to).await?)
    }

    /// Get the current position.
    pub async fn stream_position(&mut self) -> Result<u64> {
        if cfg!(feature = "tokio-compat-flush") {
            self.stream.flush().await?;
        }
        Ok(self.stream.stream_position().await?)
    }

    /// Get the length of this stream by seeking to the end
    /// and then restoring the previous cursor position.
    pub async fn len(&mut self) -> Result<u64> {
        stream_length(&mut self.stream).await
    }

    /// Write a length-prefixed `String` to the stream.
    pub async fn write_string<S: AsRef<str>>(
        &mut self,
        value: S,
    ) -> Result<usize> {
        let bytes = value.as_ref().as_bytes();
        guard_size!(bytes.len(), self.options.max_buffer_size);
        if cfg!(feature = "64bit") {
            self.write_u64(bytes.len() as u64).await?;
        } else {
            self.write_u32(bytes.len() as u32).await?;
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

    /// Flush the write buffer.
    pub async fn flush(&mut self) -> Result<()> {
        self.stream.flush().await
    }
}

/// Trait for encoding to binary.
#[async_trait]
pub trait Encodable {
    /// Encode self into the binary writer.
    async fn encode<W: AsyncWrite + AsyncSeek + Unpin + Send>(
        &self,
        writer: &mut BinaryWriter<W>,
    ) -> Result<()>;
}

/// Trait for decoding from binary.
#[async_trait]
pub trait Decodable {
    /// Decode from the binary reader into self.
    async fn decode<R: AsyncRead + AsyncSeek + Unpin + Send>(
        &mut self,
        reader: &mut BinaryReader<R>,
    ) -> Result<()>;
}

/// Encode to a binary buffer.
pub async fn encode(
    encodable: &impl Encodable,
    options: Options,
) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut stream = BufWriter::new(Cursor::new(&mut buffer));
    encode_stream(encodable, &mut stream, options).await?;
    Ok(buffer)
}

/// Decode from a binary buffer.
pub async fn decode<T: Decodable + Default>(
    buffer: &[u8],
    options: Options,
) -> Result<T> {
    let mut stream = BufReader::new(Cursor::new(buffer));
    decode_stream::<T, _>(&mut stream, options).await
}

/// Encode to a stream.
pub async fn encode_stream<S>(
    encodable: &impl Encodable,
    stream: &mut S,
    options: Options,
) -> Result<()>
where
    S: AsyncWrite + AsyncSeek + Send + Sync + Unpin,
{
    let mut writer = BinaryWriter::new(stream, options);
    encodable.encode(&mut writer).await?;
    writer.flush().await?;
    Ok(())
}

/// Decode from a stream.
pub async fn decode_stream<
    T: Decodable + Default,
    S: AsyncRead + AsyncSeek + Send + Sync + Unpin,
>(
    stream: &mut S,
    options: Options,
) -> Result<T> {
    let mut reader = BinaryReader::new(stream, options);
    let mut decoded: T = T::default();
    decoded.decode(&mut reader).await?;
    Ok(decoded)
}

#[async_trait]
impl<T> Encodable for Option<T>
where
    T: Encodable + Default + Send + Sync,
{
    async fn encode<W: AsyncWrite + AsyncSeek + Unpin + Send>(
        &self,
        writer: &mut BinaryWriter<W>,
    ) -> Result<()> {
        writer.write_bool(self.is_some()).await?;
        if let Some(value) = self {
            value.encode(&mut *writer).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T> Decodable for Option<T>
where
    T: Decodable + Default + Send + Sync,
{
    async fn decode<R: AsyncRead + AsyncSeek + Unpin + Send>(
        &mut self,
        reader: &mut BinaryReader<R>,
    ) -> Result<()> {
        let has_value = reader.read_bool().await?;
        if has_value {
            let mut value: T = Default::default();
            value.decode(&mut *reader).await?;
            *self = Some(value);
        }
        Ok(())
    }
}

macro_rules! impl_encode_decode {
    ($type:ty, $read:ident, $write:ident) => {
        #[async_trait]
        impl Encodable for $type {
            async fn encode<W: AsyncWrite + AsyncSeek + Unpin + Send>(
                &self,
                writer: &mut BinaryWriter<W>,
            ) -> Result<()> {
                writer.$write(self).await?;
                Ok(())
            }
        }

        #[async_trait]
        impl Decodable for $type {
            async fn decode<R: AsyncRead + AsyncSeek + Unpin + Send>(
                &mut self,
                reader: &mut BinaryReader<R>,
            ) -> Result<()> {
                *self = reader.$read().await?;
                Ok(())
            }
        }
    };
}

impl_encode_decode!(u8, read_u8, write_u8);
impl_encode_decode!(u16, read_u16, write_u16);
impl_encode_decode!(u32, read_u32, write_u32);
impl_encode_decode!(u64, read_u64, write_u64);
impl_encode_decode!(u128, read_u128, write_u128);
impl_encode_decode!(usize, read_usize, write_usize);

impl_encode_decode!(i8, read_i8, write_i8);
impl_encode_decode!(i16, read_i16, write_i16);
impl_encode_decode!(i32, read_i32, write_i32);
impl_encode_decode!(i64, read_i64, write_i64);
impl_encode_decode!(i128, read_i128, write_i128);
impl_encode_decode!(isize, read_isize, write_isize);

impl_encode_decode!(f32, read_f32, write_f32);
impl_encode_decode!(f64, read_f64, write_f64);

impl_encode_decode!(bool, read_bool, write_bool);
impl_encode_decode!(char, read_char, write_char);
impl_encode_decode!(String, read_string, write_string);

#[cfg(test)]
mod test {
    use super::{BinaryReader, BinaryWriter, Decodable, Encodable};
    use anyhow::Result;
    use async_trait::async_trait;
    use futures::io::{
        AsyncRead, AsyncSeek, AsyncWrite, BufReader, BufWriter, Cursor,
    };
    use std::io::{self, SeekFrom};
    use tokio::fs::File;
    use tokio_util::compat::{
        TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt,
    };

    #[derive(Debug, Default, Eq, PartialEq)]
    struct Group(pub Vec<u8>, pub Vec<u8>);
    #[derive(Debug, Default, Eq, PartialEq)]
    struct Entry([u8; 16], Group);

    #[cfg_attr(target_arch="wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl Encodable for Group {
        async fn encode<W: AsyncWrite + AsyncSeek + Unpin + Send>(
            &self,
            writer: &mut BinaryWriter<W>,
        ) -> io::Result<()> {
            writer.write_u32(self.0.len() as u32).await?;
            writer.write_bytes(self.0.as_slice()).await?;
            writer.write_u32(self.1.len() as u32).await?;
            writer.write_bytes(self.1.as_slice()).await?;
            Ok(())
        }
    }

    #[cfg_attr(target_arch="wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl Decodable for Group {
        async fn decode<R: AsyncRead + AsyncSeek + Unpin + Send>(
            &mut self,
            reader: &mut BinaryReader<R>,
        ) -> io::Result<()> {
            let len = reader.read_u32().await?;
            self.0 = reader.read_bytes(len as usize).await?;
            let len = reader.read_u32().await?;
            self.1 = reader.read_bytes(len as usize).await?;
            Ok(())
        }
    }

    #[cfg_attr(target_arch="wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl Encodable for Entry {
        async fn encode<W: AsyncWrite + AsyncSeek + Unpin + Send>(
            &self,
            writer: &mut BinaryWriter<W>,
        ) -> io::Result<()> {
            // Write the UUID
            writer.write_bytes(self.0.as_ref()).await?;

            let size_pos = writer.stream_position().await?;

            writer.write_u32(0).await?;

            self.1.encode(&mut *writer).await?;

            // Encode the data length for lazy iteration
            let row_pos = writer.stream_position().await?;
            let row_len = row_pos - (size_pos + 4);
            writer.seek(SeekFrom::Start(size_pos)).await?;
            writer.write_u32(row_len as u32).await?;
            writer.seek(SeekFrom::Start(row_pos)).await?;

            Ok(())
        }
    }

    #[cfg_attr(target_arch="wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl Decodable for Entry {
        async fn decode<R: AsyncRead + AsyncSeek + Unpin + Send>(
            &mut self,
            reader: &mut BinaryReader<R>,
        ) -> io::Result<()> {
            let id = reader.read_bytes(16).await?.try_into().unwrap();
            self.0 = id;

            // Read in the length of the data blob
            let _ = reader.read_u32().await?;

            let mut group: Group = Default::default();
            group.decode(&mut *reader).await?;
            self.1 = group;

            Ok(())
        }
    }

    #[tokio::test]
    async fn async_tokio_encode_decode() -> Result<()> {
        let entry = Entry([0u8; 16], Group(vec![0u8; 512], vec![0u8; 512]));

        let path = "target/encode_decode.test";

        let mut file = File::create(path).await?.compat_write();

        let mut writer = BinaryWriter::new(&mut file, Default::default());
        entry.encode(&mut writer).await?;
        writer.flush().await?;

        let mut file = File::open(path).await?.compat();
        let mut reader = BinaryReader::new(&mut file, Default::default());

        let mut decoded_entry: Entry = Default::default();
        decoded_entry.decode(&mut reader).await?;

        assert_eq!(entry, decoded_entry);

        Ok(())
    }

    #[tokio::test]
    async fn async_tokio_file() -> Result<()> {
        let mock_str = "mock value".to_string();
        let mock_char = 'c';

        let write_file =
            tokio::fs::File::create("target/async-tokio.test").await?;
        let mut write_file = write_file.compat_write();
        let mut writer =
            BinaryWriter::new(&mut write_file, Default::default());
        writer.write_string(&mock_str).await?;
        writer.write_char(&mock_char).await?;
        writer.flush().await?;

        let read_file =
            tokio::fs::File::open("target/async-tokio.test").await?;
        let mut read_file = read_file.compat();
        let mut reader =
            BinaryReader::new(&mut read_file, Default::default());

        let str_value = reader.read_string().await?;
        assert_eq!(mock_str, str_value);
        let char_value = reader.read_char().await?;
        assert_eq!(mock_char, char_value);

        Ok(())
    }

    #[tokio::test]
    async fn async_tokio_memory() -> Result<()> {
        let mut buffer = Vec::new();
        let mut stream = BufWriter::new(Cursor::new(&mut buffer));
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let u_8 = 8u8;
        let u_16 = 16u16;
        let u_32 = 32u32;
        let u_64 = 64u64;
        let u_128 = 128u128;
        let u_size = 256usize;

        let i_8 = 8i8;
        let i_16 = 16i16;
        let i_32 = 32i32;
        let i_64 = 64i64;
        let i_128 = 128i128;
        let i_size = 256isize;

        let mock_str = "mock value".to_string();
        let mock_char = 'c';
        let mock_true = true;
        let mock_false = false;

        let f_32 = 3.14f32;
        let f_64 = 3.14f64;

        writer.write_u8(u_8).await?;
        writer.write_u16(u_16).await?;
        writer.write_u32(u_32).await?;
        writer.write_u64(u_64).await?;
        writer.write_u128(u_128).await?;
        writer.write_usize(u_size).await?;

        writer.write_i8(i_8).await?;
        writer.write_i16(i_16).await?;
        writer.write_i32(i_32).await?;
        writer.write_i64(i_64).await?;
        writer.write_i128(i_128).await?;
        writer.write_isize(i_size).await?;

        writer.write_string(&mock_str).await?;
        writer.write_char(mock_char).await?;

        writer.write_bool(mock_true).await?;
        writer.write_bool(mock_false).await?;

        writer.write_f32(f_32).await?;
        writer.write_f64(f_64).await?;

        writer.flush().await?;

        let mut stream = BufReader::new(Cursor::new(&mut buffer));
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        assert_eq!(u_8, reader.read_u8().await?);
        assert_eq!(u_16, reader.read_u16().await?);
        assert_eq!(u_32, reader.read_u32().await?);
        assert_eq!(u_64, reader.read_u64().await?);
        assert_eq!(u_128, reader.read_u128().await?);
        assert_eq!(u_size, reader.read_usize().await?);

        assert_eq!(i_8, reader.read_i8().await?);
        assert_eq!(i_16, reader.read_i16().await?);
        assert_eq!(i_32, reader.read_i32().await?);
        assert_eq!(i_64, reader.read_i64().await?);
        assert_eq!(i_128, reader.read_i128().await?);
        assert_eq!(i_size, reader.read_isize().await?);

        assert_eq!(mock_str, reader.read_string().await?);
        assert_eq!(mock_char, reader.read_char().await?);

        assert_eq!(mock_true, reader.read_bool().await?);
        assert_eq!(mock_false, reader.read_bool().await?);

        assert_eq!(f_32, reader.read_f32().await?);
        assert_eq!(f_64, reader.read_f64().await?);

        Ok(())
    }

    // Tests encoding and decoding using the blanket implementations
    // for primitive types provided by the macro.
    #[tokio::test]
    async fn async_encode_decode_primitives() -> Result<()> {
        let mut buffer = Vec::new();
        let mut stream = BufWriter::new(Cursor::new(&mut buffer));
        let mut writer = BinaryWriter::new(&mut stream, Default::default());

        let u_8 = 8u8;
        let u_16 = 16u16;
        let u_32 = 32u32;
        let u_64 = 64u64;
        let u_128 = 128u128;
        let u_size = 256usize;

        let i_8 = 8i8;
        let i_16 = 16i16;
        let i_32 = 32i32;
        let i_64 = 64i64;
        let i_128 = 128i128;
        let i_size = 256isize;

        let mock_str = "mock value".to_string();
        let mock_char = 'c';
        let mock_true = true;
        let mock_false = false;

        let f_32 = 3.14f32;
        let f_64 = 3.14f64;

        u_8.encode(&mut writer).await?;
        u_16.encode(&mut writer).await?;
        u_32.encode(&mut writer).await?;
        u_64.encode(&mut writer).await?;
        u_128.encode(&mut writer).await?;
        u_size.encode(&mut writer).await?;

        i_8.encode(&mut writer).await?;
        i_16.encode(&mut writer).await?;
        i_32.encode(&mut writer).await?;
        i_64.encode(&mut writer).await?;
        i_128.encode(&mut writer).await?;
        i_size.encode(&mut writer).await?;

        mock_str.encode(&mut writer).await?;
        mock_char.encode(&mut writer).await?;
        mock_true.encode(&mut writer).await?;
        mock_false.encode(&mut writer).await?;

        f_32.encode(&mut writer).await?;
        f_64.encode(&mut writer).await?;

        writer.flush().await?;

        let mut stream = BufReader::new(Cursor::new(&mut buffer));
        let mut reader = BinaryReader::new(&mut stream, Default::default());

        let mut o_u8 = 0u8;
        let mut o_u16 = 0u16;
        let mut o_u32 = 0u32;
        let mut o_u64 = 0u64;
        let mut o_u128 = 0u128;
        let mut o_usize = 0usize;

        let mut o_i8 = 0i8;
        let mut o_i16 = 0i16;
        let mut o_i32 = 0i32;
        let mut o_i64 = 0i64;
        let mut o_i128 = 0i128;
        let mut o_isize = 0isize;

        let mut o_mock_str = "".to_string();
        let mut o_mock_char = 'z';
        let mut o_mock_true = false;
        let mut o_mock_false = true;

        let mut o_f32 = 0f32;
        let mut o_f64 = 0f64;

        o_u8.decode(&mut reader).await?;
        o_u16.decode(&mut reader).await?;
        o_u32.decode(&mut reader).await?;
        o_u64.decode(&mut reader).await?;
        o_u128.decode(&mut reader).await?;
        o_usize.decode(&mut reader).await?;

        o_i8.decode(&mut reader).await?;
        o_i16.decode(&mut reader).await?;
        o_i32.decode(&mut reader).await?;
        o_i64.decode(&mut reader).await?;
        o_i128.decode(&mut reader).await?;
        o_isize.decode(&mut reader).await?;

        o_mock_str.decode(&mut reader).await?;
        o_mock_char.decode(&mut reader).await?;
        o_mock_true.decode(&mut reader).await?;
        o_mock_false.decode(&mut reader).await?;

        o_f32.decode(&mut reader).await?;
        o_f64.decode(&mut reader).await?;

        assert_eq!(u_8, o_u8);
        assert_eq!(u_16, o_u16);
        assert_eq!(u_32, o_u32);
        assert_eq!(u_64, o_u64);
        assert_eq!(u_128, o_u128);
        assert_eq!(u_size, o_usize);

        assert_eq!(i_8, o_i8);
        assert_eq!(i_16, o_i16);
        assert_eq!(i_32, o_i32);
        assert_eq!(i_64, o_i64);
        assert_eq!(i_128, o_i128);
        assert_eq!(i_size, o_isize);

        assert_eq!(mock_str, o_mock_str);
        assert_eq!(mock_char, o_mock_char);
        assert_eq!(mock_true, o_mock_true);
        assert_eq!(mock_false, o_mock_false);

        assert_eq!(f_32, o_f32);
        assert_eq!(f_64, o_f64);

        Ok(())
    }
}
