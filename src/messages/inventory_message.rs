use super::{
    message_header::MessageHeader,
};

use crate::connections::{
    type_identifier::TypeIdentifier,
};

use crate::block_structure::hash::{
    HashType,
};

use std::io::Read;

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

    pub fn deserialize_message(
        stream: &mut dyn Read, 
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> 
    {
        let mut buffer: Vec<u8> = vec![0; message_header.payload_size as usize];

        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorWhileReading);
        }
        
        let mut buffer: &[u8] = &buffer[..];

        Ok(InventoryMessage::deserialize(&mut buffer)?)
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
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let type_identifier = TypeIdentifier::deserialize(stream)?;
        let hash_value = HashType::deserialize(stream)?;
        
        Ok(InventoryMessage { 
            type_identifier, 
            hash_value 
        })
    }
}