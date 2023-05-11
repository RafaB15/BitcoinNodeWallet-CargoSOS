use super::error_message::ErrorMessage;
use std::io::Write;

use chrono::{
    DateTime,
    offset::Utc,
};

pub trait Serializable {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage>;    
}

impl Serializable for i32 {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for u8 {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for u16 {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for u32 {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for u64 {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for Vec<u8> {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for [u8] {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for bool {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        let boolean: [u8; 1] = match self {
            true => [0x01],
            false => [0x00],
        };

        match stream.write(&boolean) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for String {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.as_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Serializable for DateTime<Utc> {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.timestamp().to_le_bytes())  {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{
        Serializable,
        ErrorMessage,
    };

    use chrono::{
        DateTime,
        offset::Utc,
        NaiveDateTime,
    };

    #[test]
    fn test01_serialize_correctly_i32() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0x5C, 0x06, 0x00, 0x00];
        
        let mut stream: Vec<u8> = Vec::new();
        let numero: i32 = 1628;

        numero.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test02_serialize_correctly_u8() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0x54];
        
        let mut stream: Vec<u8> = Vec::new();
        let numero: u8 = 84;

        numero.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }
    
    #[test]
    fn test03_serialize_correctly_u16() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0x9E, 0x3F];
        
        let mut stream: Vec<u8> = Vec::new();
        let numero: u16 = 16286;

        numero.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test04_serialize_correctly_u32() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0xAD, 0x83, 0xF8, 0x00];
        
        let mut stream: Vec<u8> = Vec::new();
        let numero: u32 = 16_286_637;

        numero.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test05_serialize_correctly_u64() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19, 0x00, 0x00, 0x00];
        
        let mut stream: Vec<u8> = Vec::new();
        let numero: u64 = 1111_1111_1111;

        numero.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test06_serialize_correctly_vec() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19];
        
        let mut stream: Vec<u8> = Vec::new();
        let vector: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19];

        vector.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test07_serialize_correctly_array() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0xC7, 0x01, 0xBD, 0xDE, 0x19];
        
        let mut stream: Vec<u8> = Vec::new();
        let vector: [u8; 5] = [0xC7, 0x01, 0xBD, 0xDE, 0x19];

        vector.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test08_serialize_correctly_bool() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0x01];
        
        let mut stream: Vec<u8> = Vec::new();
        let boolean: bool = true;

        boolean.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        let stream_esperado: Vec<u8> = vec![0x00];
        
        let mut stream: Vec<u8> = Vec::new();
        let boolean: bool = false;

        boolean.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test09_serialize_correctly_string() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0x62, 0x75, 0x75];
        
        let mut stream: Vec<u8> = Vec::new();
        let string: String = "buu".to_string();

        string.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test10_serialize_correctly_datetime() -> Result<(), ErrorMessage> {
        
        let stream_esperado: Vec<u8> = vec![0x5C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        
        let mut stream: Vec<u8> = Vec::new();
        let naive = NaiveDateTime::from_timestamp_opt(1628, 0).unwrap();
        let date: DateTime<Utc> = DateTime::<Utc>::from_utc(naive, Utc);

        date.serialize(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

}