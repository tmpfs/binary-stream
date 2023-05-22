use anyhow::Result;
use binary_stream::{BinaryReader, BinaryWriter, Endian};
use std::io::Cursor;
use tempfile::tempfile;

#[test]
fn borrow_test() -> Result<()> {
    let mut buffer = Vec::new();
    let stream = Cursor::new(&mut buffer);

    let mut writer = BinaryWriter::new(stream, Endian::Big);
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
    let mut reader = BinaryReader::new(&mut stream, Endian::Big);

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

    let mut writer = BinaryWriter::new(&mut stream, Endian::Big);
    writer.write_u32(42)?;
    writer.write_string("foo")?;
    writer.write_char('b')?;

    let mut buffer = stream.into_inner();

    if cfg!(feature = "32bit") {
        assert_eq!(15, buffer.len());
    } else {
        assert_eq!(19, buffer.len());
    }

    let mut stream = Cursor::new(&mut buffer);
    let mut reader = BinaryReader::new(&mut stream, Endian::Big);

    reader.seek(0)?;
    let value = reader.read_u32()?;
    assert_eq!(42, value);

    assert_eq!(4, reader.tell()?);

    let value = reader.read_string()?;
    assert_eq!("foo", &value);

    let value = reader.read_char()?;
    assert_eq!('b', value);

    if cfg!(feature = "32bit") {
        assert_eq!(15, buffer.len());
    } else {
        assert_eq!(19, buffer.len());
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
    writer.seek(seek_loc)?;
    assert_eq!(writer.tell()?, seek_loc);
    writer.write_f32(temp)?;

    let mut reader = BinaryReader::new(&mut file, Default::default());
    reader.seek(seek_loc)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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

    writer.seek(0)?;
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
    reader.seek(0)?;
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

    writer.seek(0)?;
    writer.write_f32(4.0)?;
    writer.write_f32(5.0)?;
    writer.write_f32(6.0)?;

    let mut reader = BinaryReader::new(&mut stream, Default::default());
    reader.seek(0)?;
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

    writer.seek(0)?;
    writer.write_f32(4.0)?;
    writer.write_f32(5.0)?;
    writer.write_f32(6.0)?;

    //let file = std::fs::File::open("filestream_overlapping.test")?;

    writer.seek(0)?;
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
