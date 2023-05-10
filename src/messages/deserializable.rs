use std::io::Read;
use super::error_message::ErrorMessage;

pub trait Deserializable {
    type Value;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage>;
}
