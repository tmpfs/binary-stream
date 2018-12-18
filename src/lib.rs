extern crate bincode;

use bincode::{deserialize, deserialize_from, serialize};
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;

pub enum OpenType {
    OpenAndCreate,
    Open,
}

pub struct BinaryReader {
    file: fs::File,
}

impl BinaryReader {
    pub fn new(filepath: &str, open_type: OpenType) -> BinaryReader {
        let mut file;

        match open_type {
            OpenType::OpenAndCreate => file = fs::File::create(filepath),
            OpenType::Open => file = fs::File::open(filepath),
        }

        if !file.is_ok() {
            panic!("Failed to open file: {}", filepath);
        }

        let file = file.unwrap();

        BinaryReader { file: file }
    }

    pub fn seek_to(&mut self, position: u64) -> u64 {
        let result = self.file.seek(SeekFrom::Start(position));

        if !result.is_ok() {
            panic!(result.err().unwrap());
        }

        result.unwrap()
    }

    pub fn get_cur_pos(&mut self) -> u64 {
        let result = self.file.seek(SeekFrom::Current(0));

        if !result.is_ok() {
            panic!(result.err().unwrap());
        }

        result.unwrap()
    }

    pub fn read_string(&mut self) -> String {
        deserialize_from(&self.file).unwrap()
    }

    pub fn read_f32(&mut self) -> f32 {
        let mut buffer: Vec<u8> = vec![0; 8];

        self.file.read(&mut buffer).unwrap();

        let value: f32 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_f64(&mut self) -> f64 {
        let mut buffer: Vec<u8> = vec![0; 8];

        self.file.read(&mut buffer).unwrap();

        let value: f64 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_isize(&mut self) -> isize {
        let mut buffer: Vec<u8> = vec![0; 8];

        self.file.read(&mut buffer).unwrap();

        let value: isize = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_usize(&mut self) -> usize {
        let mut buffer: Vec<u8> = vec![0; 8];

        self.file.read(&mut buffer).unwrap();

        let value: usize = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_u64(&mut self) -> u64 {
        let mut buffer: Vec<u8> = vec![0; 8];

        self.file.read(&mut buffer).unwrap();

        let value: u64 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_i64(&mut self) -> i64 {
        let mut buffer: Vec<u8> = vec![0; 8];

        self.file.read(&mut buffer).unwrap();

        let value: i64 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_u32(&mut self) -> u32 {
        let mut buffer: Vec<u8> = vec![0; 4];

        self.file.read(&mut buffer).unwrap();

        let value: u32 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_i32(&mut self) -> i32 {
        let mut buffer: Vec<u8> = vec![0; 4];

        self.file.read(&mut buffer).unwrap();

        let value: i32 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_u16(&mut self) -> u16 {
        let mut buffer: Vec<u8> = vec![0; 2];

        self.file.read(&mut buffer).unwrap();

        let value: u16 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_i16(&mut self) -> i16 {
        let mut buffer: Vec<u8> = vec![0; 2];

        self.file.read(&mut buffer).unwrap();

        let value: i16 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_u8(&mut self) -> u8 {
        let mut buffer: Vec<u8> = vec![0; 1];

        self.file.read(&mut buffer).unwrap();

        let value: u8 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_i8(&mut self) -> i8 {
        let mut buffer: Vec<u8> = vec![0; 1];

        self.file.read(&mut buffer).unwrap();

        let value: i8 = deserialize(&buffer).unwrap();

        value
    }

    pub fn read_bytes(&mut self, length: u64) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![0; length as usize];
        self.file.read(&mut buffer).unwrap();

        buffer
    }
}

pub struct BinaryWriter {
    file: fs::File,
}

impl BinaryWriter {
    pub fn new(filepath: &str, open_type: OpenType) -> BinaryWriter {
        let mut file;

        match open_type {
            OpenType::OpenAndCreate => file = fs::File::create(filepath),
            OpenType::Open => file = fs::File::open(filepath),
        }

        if !file.is_ok() {
            panic!("Failed to open file: {}", filepath);
        }

        let file = file.unwrap();

        BinaryWriter { file: file }
    }

    pub fn seek_to(&mut self, position: u64) -> u64 {
        let result = self.file.seek(SeekFrom::Start(position));

        if !result.is_ok() {
            panic!(result.err().unwrap());
        }

        result.unwrap()
    }

    pub fn get_cur_pos(&mut self) -> u64 {
        let result = self.file.seek(SeekFrom::Current(0));

        if !result.is_ok() {
            panic!(result.err().unwrap());
        }

        result.unwrap()
    }

    pub fn write_string(&mut self, value: String) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_f32(&mut self, value: f32) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_f64(&mut self, value: f64) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_isize(&mut self, value: isize) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_usize(&mut self, value: usize) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_u64(&mut self, value: u64) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_i64(&mut self, value: i64) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_u32(&mut self, value: u32) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_i32(&mut self, value: i32) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_u16(&mut self, value: u16) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_i16(&mut self, value: i16) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_u8(&mut self, value: u8) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_i8(&mut self, value: i8) {
        let data: Vec<u8> = serialize(&value).unwrap();
        self.file.write(&data).unwrap();
    }

    pub fn write_bytes(&mut self, data: Vec<u8>) {
        self.file.write(&data).unwrap();
    }
}