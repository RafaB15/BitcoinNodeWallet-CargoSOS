use super::error_serialization::ErrorSerialization;
use std::io::Read;

pub trait DeserializableInternalOrder: Sized {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization>;
}

impl<const N: usize> DeserializableInternalOrder for [u8; N] {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; N];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(format!(
                "Deserializing [u8; {N}]",
            )));
        }

        Ok(buffer)
    }
}
