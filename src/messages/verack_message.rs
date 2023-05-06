use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;

pub const VERACK_TYPE: [u8; 12] = [118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0];

pub struct VerackMessage {}

impl VerackMessage {

    pub fn new() -> Self {
        VerackMessage {  }
    }
}

impl Serializable for VerackMessage {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        
        // message_type: [u8; 12]
        if stream.write(&VERACK_TYPE).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        // payload_size: u32
        let payload_size: u32 = 0;
        if stream.write(&payload_size.to_be_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }
        
        // checksum: [u8; 4]
        let payload = [0u8; 0];
        let hash_of_bytes = sha256d::Hash::hash(&payload);

        let hash_bytes: &[u8] = hash_of_bytes.as_ref();
        let hash_bytes: &[u8; 4] = match hash_bytes.try_into() {
            Ok(hash_bytes) => hash_bytes,
            _ => return Err(ErrorMessage::ErrorInSerialization),
        };

        if stream.write(hash_bytes).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        Ok(())
    }
}

impl Deserializable for VerackMessage {
    
    type Value = Self;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        todo!()
    }
}