use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;

pub const VERACK_TYPE: [u8; 12] = [118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0];

#[derive(Debug, std::cmp::PartialEq)]
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
            _ => return Err(ErrorMessage::ErrorInDeserialization),
        };

        if receive_checksum != *checksum {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        Ok(VerackMessage::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::{
        serializable::Serializable, 
        deserializable::Deserializable
    };

    use super::VerackMessage;


    #[test]
    fn test01_serializar() {
        let message_verack = VerackMessage::new();
        let mut stream: [u32; 1] = [10];

        message_verack.serialize(&mut stream);

        assert_eq!([10], stream);
    }

    #[test]
    fn test02_deserializar() {
        let mut stream = "".as_bytes();
        let verack_esperado = VerackMessage::new();

        let result_message_verack = VerackMessage::deserialize(&mut stream);

        assert_eq!(Ok(verack_esperado), result_message_verack);
    }

}