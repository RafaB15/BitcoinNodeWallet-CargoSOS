use super::{
    command_name::CommandName, compact_size::CompactSize, inventory_vector::InventoryVector,
    message::Message,
};

use std::io::Read;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

pub struct InventoryMessage {
    pub inventory_vectors: Vec<InventoryVector>,
}

impl InventoryMessage {
    pub fn new(inventory_vectors: Vec<InventoryVector>) -> InventoryMessage {
        InventoryMessage { inventory_vectors }
    }
}

impl Message for InventoryMessage {
    fn get_command_name() -> CommandName {
        CommandName::Inventory
    }
}

impl SerializableInternalOrder for InventoryMessage {
    fn io_serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        CompactSize::new(self.inventory_vectors.len() as u64).le_serialize(stream)?;
        for inventory_vector in &self.inventory_vectors {
            inventory_vector.io_serialize(stream)?;
        }

        Ok(())
    }
}

impl DeserializableInternalOrder for InventoryMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let count = CompactSize::le_deserialize(stream)?.value;
        let mut inventory_vectors = Vec::new();
        for _ in 0..count {
            inventory_vectors.push(InventoryVector::io_deserialize(stream)?);
        }

        Ok(InventoryMessage { inventory_vectors })
    }
}
