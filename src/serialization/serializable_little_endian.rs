use super::error_serialization::ErrorSerialization;
use std::io::Write;

use chrono::{offset::Utc, DateTime};

/// This trait is used to serializing from a stream in little endian
///
/// ### Error
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when there is an error in the serialization
pub trait SerializableLittleEndian {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization>;
}

impl SerializableLittleEndian for i32 {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing i32".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for i64 {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing i64".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for u8 {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let bytes = [*self];
        let bytes: &[u8] = bytes.as_slice();
        match stream.write(bytes) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing u8".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for u16 {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing u16".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for u32 {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing u32".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for u64 {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing u64".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for Vec<u8> {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let mut little_endian: Vec<u8> = Vec::new();
        for byte in self.iter().rev() {
            little_endian.push(*byte);
        }

        match stream.write(little_endian.as_slice()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing Vec<u8>".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for [u8] {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let mut little_endian: Vec<u8> = Vec::new();
        for byte in self.iter().rev() {
            little_endian.push(*byte);
        }

        match stream.write(little_endian.as_slice()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing [u8]".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for bool {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        let boolean: [u8; 1] = match self {
            true => [0x01],
            false => [0x00],
        };

        match stream.write(&boolean) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing bool".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for String {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(self.as_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing String".to_string(),
            )),
        }
    }
}

impl SerializableLittleEndian for DateTime<Utc> {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.timestamp().to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing DateTime<Utc>".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{ErrorSerialization, SerializableLittleEndian};

    use chrono::{offset::Utc, DateTime, NaiveDateTime};

    #[test]
    fn test01_serialize_correctly_i32() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x5C, 0x06, 0x00, 0x00];

        let mut stream: Vec<u8> = Vec::new();
        let number: i32 = 1628;

        number.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_serialize_correctly_u8() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x54];

        let mut stream: Vec<u8> = Vec::new();
        let number: u8 = 84;

        number.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test03_serialize_correctly_u16() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x9E, 0x3F];

        let mut stream: Vec<u8> = Vec::new();
        let number: u16 = 16286;

        number.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test04_serialize_correctly_u32() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0xAD, 0x83, 0xF8, 0x00];

        let mut stream: Vec<u8> = Vec::new();
        let number: u32 = 16_286_637;

        number.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test05_serialize_correctly_u64() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19, 0x00, 0x00, 0x00];

        let mut stream: Vec<u8> = Vec::new();
        let number: u64 = 1111_1111_1111;

        number.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test06_serialize_correctly_vec() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19];

        let mut stream: Vec<u8> = Vec::new();
        let vector: Vec<u8> = vec![0x19, 0xDE, 0xBD, 0x01, 0xC7];

        vector.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test07_serialize_correctly_array() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19];

        let mut stream: Vec<u8> = Vec::new();
        let vector: [u8; 5] = [0x19, 0xDE, 0xBD, 0x01, 0xC7];

        vector.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test08_serialize_correctly_bool() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x01];

        let mut stream: Vec<u8> = Vec::new();
        let boolean: bool = true;

        boolean.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        let expected_stream: Vec<u8> = vec![0x00];

        let mut stream: Vec<u8> = Vec::new();
        let boolean: bool = false;

        boolean.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test09_serialize_correctly_string() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x62, 0x75, 0x75];

        let mut stream: Vec<u8> = Vec::new();
        let string: String = "buu".to_string();

        string.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test10_serialize_correctly_datetime() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x5C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let mut stream: Vec<u8> = Vec::new();
        let naive = NaiveDateTime::from_timestamp_opt(1628, 0).unwrap();
        let date: DateTime<Utc> = DateTime::<Utc>::from_utc(naive, Utc);

        date.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }
}
