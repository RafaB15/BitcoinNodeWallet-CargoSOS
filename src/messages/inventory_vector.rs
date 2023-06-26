use crate::connections::type_identifier::TypeIdentifier;

use crate::block_structure::hash::HashType;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use std::io::Read;

/// It's the reduce representation of any sendable data
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryVector {
    pub type_identifier: TypeIdentifier,
    pub hash_value: HashType,
}

impl InventoryVector {
    pub fn new(type_identifier: TypeIdentifier, hash_value: HashType) -> InventoryVector {
        InventoryVector {
            type_identifier,
            hash_value,
        }
    }
}

impl SerializableInternalOrder for InventoryVector {
    fn io_serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        self.type_identifier.le_serialize(stream)?;
        self.hash_value.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for InventoryVector {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(InventoryVector {
            type_identifier: TypeIdentifier::le_deserialize(stream)?,
            hash_value: HashType::le_deserialize(stream)?,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::connections::type_identifier::TypeIdentifier;

    use super::*;

    #[test]
    fn test_01_correct_inventory_vector_serialization() {
        let mut serialized_fields = vec![];
        let type_identifier = TypeIdentifier::Block;
        let hash_value = [
            0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80,
            0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03,
            0x98, 0xa1, 0x4f, 0x3f,
        ];

        type_identifier
            .le_serialize(&mut serialized_fields)
            .unwrap();
        hash_value.le_serialize(&mut serialized_fields).unwrap();

        let inv_vector = InventoryVector {
            type_identifier,
            hash_value,
        };

        let mut serialized_vector = vec![];
        inv_vector.io_serialize(&mut serialized_vector).unwrap();

        assert_eq!(serialized_fields, serialized_vector);
    }

    #[test]
    fn test_02_correct_inventory_vector_deserialization() {
        let type_identifier = TypeIdentifier::Block;
        let hash_value = [
            0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80,
            0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03,
            0x98, 0xa1, 0x4f, 0x3f,
        ];

        let inv_vec = InventoryVector {
            type_identifier,
            hash_value,
        };

        let mut serialized_message = vec![];
        inv_vec.io_serialize(&mut serialized_message).unwrap();
        let deserialized_message =
            InventoryVector::io_deserialize(&mut serialized_message.as_slice()).unwrap();

        assert_eq!(inv_vec, deserialized_message);
    }
}
