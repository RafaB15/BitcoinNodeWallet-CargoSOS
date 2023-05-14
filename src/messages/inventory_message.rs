use crate::connections::{
    type_identifier::TypeIdentifier,
};

use crate::block_structure::{
    hash::HashType,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

pub struct InventoryMessage {
    pub type_identifier: TypeIdentifier,
    pub hash_value: HashType,
}

impl InventoryMessage {

    pub fn new(type_identifier: TypeIdentifier, hash_value: HashType) -> Self {
        InventoryMessage {
            type_identifier,
            hash_value,
        }
    }
}

impl Serializable for InventoryMessage {
    
    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        self.type_identifier.serialize(stream)?;
        self.hash_value.serialize(stream)?;
        
        Ok(())
    }
}

impl Deserializable for InventoryMessage {
    
    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let type_identifier = TypeIdentifier::deserialize(stream)?;
        let hash_value: HashType = HashType::deserializable(stream)?;
        
        Ok(InventoryMessage { 
            type_identifier, 
            hash_value 
        })
    }
}