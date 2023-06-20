use super::{command_name::CommandName, message::Message};

use crate::block_structure::transaction::Transaction;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
};

use std::io::{Read, Write};

pub struct TxMessage {
    pub transaction: Transaction,
}

impl Message for TxMessage {
    fn get_command_name() -> CommandName {
        CommandName::Tx
    }
}

impl SerializableInternalOrder for TxMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.transaction.io_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for TxMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(TxMessage {
            transaction: Transaction::io_deserialize(stream)?,
        })
    }
}
