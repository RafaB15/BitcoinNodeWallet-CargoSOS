use std::io::Read;
use super::error_message::ErrorMessage;

use chrono::{
    DateTime,
    NaiveDateTime,
    offset::Utc,
};

pub trait Deserializable {
    type Value;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage>;
}

pub fn get_slice<const N: usize>(buffer: &[u8], posicion: &mut usize) -> Result<[u8; N], ErrorMessage>{
    let inicio = *posicion;
    let slice: [u8; N] = match buffer[inicio..(N + inicio)].try_into() {
        Ok(slice) => slice,
        _ => return Err(ErrorMessage::ErrorInDeserialization),
    };

    *posicion += N;
    Ok(slice)
}

impl Deserializable for i32 {
    
    type Value = i32;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(i32::from_le_bytes(buffer))
    }
}

impl Deserializable for i64 {
    
    type Value = i64;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer= [0u8; 8];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(i64::from_le_bytes(buffer))
    }
}

impl Deserializable for u8 {
    type Value = u8;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 1];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(u8::from_le_bytes(buffer))
    }
}

impl Deserializable for u16 {
    type Value = u16;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 2];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(u16::from_le_bytes(buffer))
    }
}

impl Deserializable for u32 {
    type Value = u32;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(u32::from_le_bytes(buffer))
    }
}

impl Deserializable for u64 {
    type Value = u64;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 8];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(u64::from_le_bytes(buffer))
    }
}

impl Deserializable for [u8; 4] {
    
    type Value = [u8; 4];

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer= [0u8; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(buffer)
    }
}

impl Deserializable for [u8; 12] {
    
    type Value = [u8; 12];

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 12];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(buffer)
    }
}

impl Deserializable for bool {
    type Value = bool;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 1];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        match buffer[0] {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Deserializable for DateTime<Utc> {
    type Value = DateTime<Utc>;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let timestamp_int = i64::deserialize(stream)?;
        match NaiveDateTime::from_timestamp_opt(timestamp_int, 0) {
            Some(utc_timestamp) => Ok(DateTime::<Utc>::from_utc(utc_timestamp, Utc)),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}