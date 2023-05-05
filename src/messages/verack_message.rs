use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    payload::Payload,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};

pub struct VerackMessage {}

impl VerackMessage {

    pub fn new() -> Self {
        VerackMessage {  }
    }
}

impl Payload for VerackMessage {
    fn get_message_type(&self) -> [u8; 12] {
        todo!()
    }
}

impl Serializable for VerackMessage {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        todo!()
    }
}

impl Deserializable for VerackMessage {
    type Value = Self;
    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        todo!()
    }
}