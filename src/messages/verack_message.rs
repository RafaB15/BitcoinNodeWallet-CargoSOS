use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;

pub const VERACK_TYPE: [u8; 12] = [118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0];

pub struct VerackMessage {
    pub magic_bytes: [u8; 4],
}

impl VerackMessage {
    pub fn new(magic_bytes: [u8; 4]) -> Self {
        VerackMessage { 
            magic_bytes
         }
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
        if stream.write(&payload_size.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }
        
        // checksum: [u8; 4]
        let payload = [0u8; 0];
        let hash_of_bytes = sha256d::Hash::hash(&payload);

        let hash_bytes: &[u8] = hash_of_bytes.as_ref();
        let checksum: &[u8; 4] = match (&hash_bytes[0..4]).try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorInSerialization),
        };

        if stream.write(checksum).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        Ok(())
    }
}

impl Deserializable for VerackMessage {
    
    type Value = Self;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        
        let mut num_buffer = [0u8; 4];
        if stream.read_exact(&mut num_buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        let payload_size = u32::from_be_bytes(num_buffer);

        if payload_size != 0 {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        
        let receive_checksum: [u8; 4] = [0u8; 4];
        if stream.read_exact(&mut num_buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        let payload = [0u8; 0];
        let hash_of_bytes = sha256d::Hash::hash(&payload);

        let hash_bytes: &[u8] = hash_of_bytes.as_ref();
        let checksum: &[u8; 4] = match (&hash_bytes[0..4]).try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorInSerialization),
        };

        if receive_checksum != *checksum {
            return Err(ErrorMessage::ErrorInSerialization);
        }

        Ok(VerackMessage::new())
    }
}