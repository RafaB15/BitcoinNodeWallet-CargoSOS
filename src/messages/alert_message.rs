use super::{command_name::CommandName, message::Message};

use std::io::{Read, Write};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization, serializable_big_endian::SerializableBigEndian,
    serializable_internal_order::SerializableInternalOrder,
};

#[derive(Debug)]
pub struct AlertMessage {
    contents: Vec<u8>,
}

impl Message for AlertMessage {
    fn get_command_name() -> CommandName {
        CommandName::Alert
    }
}

impl SerializableInternalOrder for AlertMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.contents.be_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for AlertMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buf = Vec::new();
        match stream.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(_) => {
                return Err(ErrorSerialization::ErrorInDeserialization(
                    "While deserializing alert message".to_string(),
                ))
            }
        }

        Ok(AlertMessage { contents: buf })
    }
}
