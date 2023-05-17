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
    deserializable_little_endian::DeserializableLittleEndian,
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

impl SerializableLittleEndian for InventoryMessage {
    
    fn le_serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        self.type_identifier.le_serialize(stream)?;
        self.hash_value.le_serialize(stream)?;
        
        Ok(())
    }
}

impl DeserializableLittleEndian for InventoryMessage {
    
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let type_identifier = TypeIdentifier::le_deserialize(stream)?;
        let hash_value = HashType::le_deserialize(stream)?;
        
        Ok(InventoryMessage { 
            type_identifier, 
            hash_value 
        })
    }
}