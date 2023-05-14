use super::error_serialization::ErrorSerialization;
use std::io::Read;

use chrono::{
    DateTime,
    NaiveDateTime,
    offset::Utc,
};

pub trait Deserializable : Sized {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization>;
}

impl Deserializable for i32 {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing i32".to_string()));
        }
        Ok(i32::from_le_bytes(buffer))
    }
}

impl Deserializable for i64 {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer= [0u8; 8];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing i64".to_string()));
        }
        Ok(i64::from_le_bytes(buffer))
    }
}

impl Deserializable for u8 {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 1];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing u8".to_string()));
        }
        Ok(u8::from_le_bytes(buffer))
    }
}

impl Deserializable for u16 {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 2];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing u16".to_string()));
        }
        Ok(u16::from_le_bytes(buffer))
    }
}

impl Deserializable for u32 {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing u32".to_string()));
        }
        Ok(u32::from_le_bytes(buffer))
    }
}

impl Deserializable for u64 {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 8];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing u64".to_string()));
        }
        Ok(u64::from_le_bytes(buffer))
    }
}

impl Deserializable for [u8; 4] {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer= [0u8; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing [u8; 4]".to_string()));
        }
        Ok(buffer)
    }
}

impl Deserializable for [u8; 12] {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 12];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing [u8; 12]".to_string()));
        }
        Ok(buffer)
    }
}

impl Deserializable for [u8; 32] {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 32];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing [u8; 12]".to_string()));
        }
        Ok(buffer)
    }
}

impl Deserializable for bool {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 1];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization("Deserializing bool".to_string()));
        }
        match buffer[0] {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(ErrorSerialization::ErrorInDeserialization(format!("The given buffer does not represent a boolean: {:?}", buffer))),
        }
    }
}

impl Deserializable for DateTime<Utc> {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let timestamp_int = i64::deserialize(stream)?;
        match NaiveDateTime::from_timestamp_opt(timestamp_int, 0) {
            Some(utc_timestamp) => Ok(DateTime::<Utc>::from_utc(utc_timestamp, Utc)),
            _ => Err(ErrorSerialization::ErrorInDeserialization("Deserializing DateTime<Utc>".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{
        Deserializable,
        ErrorSerialization,
    };

    use chrono::{
        DateTime,
        offset::Utc,
        NaiveDateTime,
    };

    #[test]
    fn test01_deserialize_correctly_i32() -> Result<(), ErrorSerialization> {

        let stream: Vec<u8> = vec![0x5C, 0x06, 0x00, 0x00];
        let mut stream: &[u8] = &stream;
        let number: i32 = 1628;

        let expected_number = i32::deserialize(&mut stream)?;

        assert_eq!(expected_number, number);

        Ok(())
    }

    #[test]
    fn test02_deserialize_correctly_i64() -> Result<(), ErrorSerialization> {

        let stream: Vec<u8> = vec![0x5C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &stream;
        let number: i64 = 1628;

        let expected_number = i64::deserialize(&mut stream)?;

        assert_eq!(expected_number, number);

        Ok(())
    }

    #[test]
    fn test03_deserialize_correctly_u8() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0x54];
        let mut stream: &[u8] = &stream;
        
        let expected_number: u8 = 84;

        let  number = u8::deserialize(&mut stream)?;

        assert_eq!(expected_number, number);

        Ok(())
    }
    
    #[test]
    fn test04_deserialize_correctly_u16() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0x9E, 0x3F];
        let mut stream: &[u8] = &stream;
        
        let expected_number: u16 = 16286;

        let number = u16::deserialize(&mut stream)?;

        assert_eq!(expected_number, number);

        Ok(())
    }

    #[test]
    fn test05_deserialize_correctly_u32() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0xAD, 0x83, 0xF8, 0x00];
        let mut stream: &[u8] = &stream;
        
        let expected_number: u32 = 16_286_637;

        let number = u32::deserialize(&mut stream)?;

        assert_eq!(expected_number, number);

        Ok(())
    }

    #[test]
    fn test06_deserialize_correctly_u64() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &stream;
        
        let expected_number: u64 = 1111_1111_1111;

        let number = u64::deserialize(&mut stream)?;

        assert_eq!(expected_number, number);

        Ok(())
    }

    #[test]
    fn test07_deserialize_correctly_array_4() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE];
        let mut stream: &[u8] = &stream;        
        
        let expected_vector: [u8; 4] = [0xC7, 0x01, 0xBD, 0xDE];

        let vector = <[u8; 4] as Deserializable>::deserialize(&mut stream)?;

        assert_eq!(expected_vector, vector);

        Ok(())
    }

    #[test]
    fn test08_deserialize_correctly_array_12() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19, 0x01, 0xBD, 0xDE, 0x19, 0x01, 0xBD, 0xDE];
        let mut stream: &[u8] = &stream;        
        
        let expected_vector: [u8; 12] = [0xC7, 0x01, 0xBD, 0xDE, 0x19, 0x01, 0xBD, 0xDE, 0x19, 0x01, 0xBD, 0xDE];

        let vector = <[u8; 12] as Deserializable>::deserialize(&mut stream)?;

        assert_eq!(expected_vector, vector);

        Ok(())
    }

    #[test]
    fn test09_deserialize_correctly_bool() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0x01];
        let mut stream: &[u8] = &stream;
        
        let expected_boolean: bool = true;

        let boolean = bool::deserialize(&mut stream)?;

        assert_eq!(expected_boolean, boolean);

        let stream: Vec<u8> = vec![0x00];
        let mut stream: &[u8] = &stream;
        
        let expected_boolean: bool = false;

        let boolean = bool::deserialize(&mut stream)?;

        assert_eq!(expected_boolean, boolean);

        Ok(())
    }

    #[test]
    fn test10_deserialize_correctly_datetime() -> Result<(), ErrorSerialization> {
        
        let stream: Vec<u8> = vec![0x5C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &stream;
        
        let naive = NaiveDateTime::from_timestamp_opt(1628, 0).unwrap();
        let expected_date: DateTime<Utc> = DateTime::<Utc>::from_utc(naive, Utc);

        let date = DateTime::deserialize(&mut stream)?;

        assert_eq!(expected_date, date);

        Ok(())
    }

}