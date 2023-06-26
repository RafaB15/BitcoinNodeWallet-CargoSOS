use super::{
    command_name::CommandName, compact_size::CompactSize, inventory_vector::InventoryVector,
    message::Message,
};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use std::io::Read;

/// It's the inventory message
#[derive(Debug, std::cmp::PartialEq)]
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

#[cfg(test)]
mod tests {

    use crate::connections::type_identifier::TypeIdentifier;

    use super::*;

    #[test]
    fn test01_correct_inv_message_serialization(){

        let mut serialized_fields = vec![];
        let inventory_vector = InventoryVector::new(
            TypeIdentifier::FilteredWitnessBlock,
            [0; 32],
        );

        let vector = vec![inventory_vector.clone()];
        CompactSize::new(vector.len() as u64).le_serialize(&mut serialized_fields).unwrap();
        inventory_vector.io_serialize(&mut serialized_fields).unwrap();

        let mut serialized_inventory_vector = vec![];
        let inventory_message = InventoryMessage::new(vec![inventory_vector]);
        inventory_message.io_serialize(&mut serialized_inventory_vector).unwrap();

        assert_eq!(serialized_fields, serialized_inventory_vector);
        
    }

    #[test]
    fn test02_correct_inv_message_deserialization(){

        let inventory_vector = InventoryVector::new(
            TypeIdentifier::FilteredWitnessBlock,
            [0; 32],
        );

        let mut serialized_inventory_vector = vec![];
        let inventory_message = InventoryMessage::new(vec![inventory_vector]);
        inventory_message.io_serialize(&mut serialized_inventory_vector).unwrap();
        let deserialized_inventory_message = InventoryMessage::io_deserialize(&mut serialized_inventory_vector.as_slice()).unwrap();

        assert_eq!(inventory_message, deserialized_inventory_message);
    }
}
