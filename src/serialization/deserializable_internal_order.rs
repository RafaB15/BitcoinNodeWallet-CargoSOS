use super::error_serialization::ErrorSerialization;
use std::io::Write;

pub trait DeserializableInternalOrder: Sized {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization>;
}

impl DeserializableInternalOrder for [u8; 32] {

    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 32];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing [u8; 32]".to_string()
            ));
        }
        
        Ok(buffer)
    }
}

impl DeserializableInternalOrder for [u8; 4] {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 4];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing [u8; 4]".to_string(),
            ));
        }
        Ok(buffer)
    }
}

impl DeserializableInternalOrder for [u8; 12] {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 12];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing [u8; 12]".to_string(),
            ));
        }

        Ok(buffer)
    }
}