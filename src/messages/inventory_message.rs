use super::{
    message::Message,
    command_name::CommandName,
};

use crate::connections::{
    type_identifier::TypeIdentifier,
};

use crate::block_structure::hash::{
    HashType,
};

use std::io::Read;

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    serializable_internal_order::SerializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};

pub struct InventoryMessage {
    pub type_identifier: TypeIdentifier,
    pub hash_value: HashType,
}

impl Message for InventoryMessage {

    fn get_command_name() -> CommandName {
        CommandName::Inventory
    }
}

impl SerializableInternalOrder for InventoryMessage {
    
    fn io_serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        self.type_identifier.le_serialize(stream)?;
        self.hash_value.le_serialize(stream)?;
        
        Ok(())
    }
}

impl DeserializableInternalOrder for InventoryMessage {
    
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let type_identifier = TypeIdentifier::le_deserialize(stream)?;
        let hash_value = HashType::le_deserialize(stream)?;
        
        Ok(InventoryMessage { 
            type_identifier, 
            hash_value 
        })
    }
}