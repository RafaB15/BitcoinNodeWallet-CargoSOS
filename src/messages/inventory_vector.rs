use crate::connections::type_identifier::TypeIdentifier;

use crate::block_structure::hash::HashType;

use std::io::Read;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

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
