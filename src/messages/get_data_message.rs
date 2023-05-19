use super::{
    message::Message,
    command_name::CommandName,
    compact_size::CompactSize,
    inventory_vector::InventoryVector,
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

pub struct GetDataMessage {
    pub inventory_vectors: Vec<InventoryVector>,
}

impl GetDataMessage {

    pub fn new(hash_vector: Vec<HashType>) -> GetDataMessage {

        let mut inventory_vectors = Vec::new();
        for hash in hash_vector {
            inventory_vectors.push(InventoryVector::new(
                TypeIdentifier::Block, 
                hash
            ));
        }

        GetDataMessage {
            inventory_vectors,
        }
    }

}

impl Message for GetDataMessage {

    fn get_command_name() -> CommandName {
        CommandName::GetData
    }
}

impl SerializableInternalOrder for GetDataMessage {
    
    fn io_serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        
        CompactSize::new(self.inventory_vectors.len() as u64).le_serialize(stream)?;
        for inventory_vector in &self.inventory_vectors {
            inventory_vector.io_serialize(stream)?;
        }

        Ok(())
    }
}

impl DeserializableInternalOrder for GetDataMessage {
    
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {

        let count = CompactSize::le_deserialize(stream)?.value;
        let mut inventory_vectors = Vec::new();
        for _ in 0..count {
            inventory_vectors.push(InventoryVector::io_deserialize(stream)?);
        }
        
        Ok(GetDataMessage { 
            inventory_vectors,
        })
    }
}