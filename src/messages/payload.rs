use super::{
    verack_message::{VerackMessage, VERACK_TYPE},
    version_message::{VersionMessage, VERSION_TYPE},
    error_message::ErrorMessage,
    deserializable::Deserializable, 
    serializable::Serializable,
};
use std::io::{Read, Write};

pub enum Payload {
    VersionMessage(VersionMessage),
    VerackMessage(VerackMessage),
}

impl Payload {
    pub fn get_from_message_type(message_type: [u8; 12], stream: &mut dyn Read) -> Result<Self, ErrorMessage> {

        match &message_type {
            VERACK_TYPE => {
                let message_payload = VerackMessage::deserialize(stream)?;
                Ok(Payload::VerackMessage(message_payload))
            },
            &VERSION_TYPE => {
                let message_payload = VersionMessage::deserialize(stream)?;
                Ok(Payload::VersionMessage(message_payload))
            },
            _ => Err(ErrorMessage::ErrorMessageUnknown),
        }
    }
}

impl Serializable for Payload {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match self {
            Payload::VersionMessage(message) => message.serialize(stream),
            Payload::VerackMessage(message) => message.serialize(stream),
        }
    }
}