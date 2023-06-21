use super::error_serialization::ErrorSerialization;
use std::io::Write;

/// This trait is used to serializing from a stream in the order was received
/// 
/// ### Error
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when there is an error in the serialization
pub trait SerializableInternalOrder {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization>;
}

impl SerializableInternalOrder for [u8] {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(self) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing [u8]".to_string(),
            )),
        }
    }
}
