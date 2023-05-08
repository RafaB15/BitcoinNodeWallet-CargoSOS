use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;

pub const VERACK_TYPE: &[u8; 12] = b"verack\0\0\0\0\0\0";
pub const VERACK_CHECKSUM: [u8; 4] = [0x5d, 0xf6, 0xe0, 0xe2];

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
        if stream.write(VERACK_TYPE).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }

        // payload_size: u32
        let payload_size: u32 = 0;
        if stream.write(&payload_size.to_le_bytes()).is_err() {
            return Err(ErrorMessage::ErrorWhileWriting);
        }
        
        // checksum: [u8; 4]
        if stream.write(&VERACK_CHECKSUM).is_err() {
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

        if receive_checksum != VERACK_CHECKSUM {
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
        let mut stream: Vec<u8> = Vec::new();

        let _ = message_verack.serialize(&mut stream);

        let stream_esperado: Vec<u8> = Vec::new();

        assert_eq!(stream_esperado, stream);
    }

    #[test]
    fn test02_deserializar() {
        let mut stream = "".as_bytes();
        let verack_esperado = VerackMessage::new();

        let result_message_verack = VerackMessage::deserialize(&mut stream);

        assert_eq!(Ok(verack_esperado), result_message_verack);
    }

}