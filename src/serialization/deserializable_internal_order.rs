use super::error_serialization::ErrorSerialization;
use std::io::Read;

/// This trait is used to deserialize from a stream in the order was received
///
/// ### Error
///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when there is an error in the deserialization
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
