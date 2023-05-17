use super::error_serialization::ErrorSerialization;
use std::io::Write;

pub trait SerializableInternalOrder {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization>;
}

impl SerializableInternal for [u8] {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(self) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing [u8]".to_string(),
            )),
        }
    }
}