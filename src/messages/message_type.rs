
use super::{
    payload::Payload,
    error_message::ErrorMessage,
};

use std::io::{Read};

pub enum MessageType {

    VersionMessage,

    VerackMessage,
}

impl MessageType {
    pub fn message_deserialize(message_type: [u8;12], stream: &mut dyn Read) -> Result<impl Payload,ErrorMessage> 
    }
}