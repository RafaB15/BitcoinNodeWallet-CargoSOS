use super::{command_name::CommandName, message::Message};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};
use std::io::{Read, Write};

/// It's the fee filter message
#[derive(Debug)]
pub struct FeeFilterMessage {
    pub feerate: u64,
}

impl Message for FeeFilterMessage {
    fn get_command_name() -> CommandName {
        CommandName::FeeFilter
    }
}

impl SerializableInternalOrder for FeeFilterMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.feerate.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for FeeFilterMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(FeeFilterMessage {
            feerate: u64::le_deserialize(stream)?,
        })
    }
}
