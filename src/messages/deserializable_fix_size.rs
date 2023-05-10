use super::error_message::ErrorMessage;
use std::io::Read;

pub trait DeserializableFixSize : Sized
{
    fn deserialize_fix_size(stream: &mut dyn Read, sizes: usize) -> Result<Self, ErrorMessage>;
}

impl DeserializableFixSize for String {

    fn deserialize_fix_size(stream: &mut dyn Read, sizes: usize) -> Result<Self, ErrorMessage> {
        let mut buffer: Vec<u8> = vec![0; sizes];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        match String::from_utf8(buffer.to_vec()) {
            Ok(user_agent) => Ok(user_agent),
            Err(_) => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}