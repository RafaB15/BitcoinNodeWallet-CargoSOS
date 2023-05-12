use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};

pub const VERACK_TYPE: &[u8; 12] = b"verack\0\0\0\0\0\0";
pub const VERACK_CHECKSUM: [u8; 4] = [0x5d, 0xf6, 0xe0, 0xe2];

const MAGIC_BYTES_SIZE: usize = 4;
const MASSAGE_TYPE_SIZE: usize = 12;
const PAYLOAD_SIZE: usize = 4;
const CHECKSUM_SIZE: usize = 4;

const HEADER_SIZE: usize = MAGIC_BYTES_SIZE + MASSAGE_TYPE_SIZE + PAYLOAD_SIZE + CHECKSUM_SIZE;

#[derive(Debug, std::cmp::PartialEq)]
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
    // magic_bytes: [u8; 4]
    // message_type: [u8; 12]
    // payload_size: u32
    // checksum: [u8; 4]
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        
        let payload_size: u32 = 0;
        let mut serialized_message = Vec::new();

        self.magic_bytes.serialize(&mut serialized_message)?;
        VERACK_TYPE.serialize(&mut serialized_message)?;
        payload_size.serialize(&mut serialized_message)?;
        VERACK_CHECKSUM.serialize(&mut serialized_message)?;

        serialized_message.serialize(stream)
    }
}

impl Deserializable for VerackMessage {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorMessage> {
        
        let mut buffer: Vec<u8> = vec![0; HEADER_SIZE];

        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorWhileReading);
        }

        let mut buffer: &[u8] = &buffer[..];

        let magic_bytes = <[u8; MAGIC_BYTES_SIZE] as Deserializable>::deserialize(&mut buffer)?;
        
        let message_type = <[u8; MASSAGE_TYPE_SIZE] as Deserializable>::deserialize(&mut buffer)?;
        if !VERACK_TYPE.eq(&message_type) {
            return Err(ErrorMessage::ErrorInDeserialization(format!("Type name not of version: {:?}", message_type)));
        }
        
        let payload_size = u32::deserialize(&mut buffer)?;
        if payload_size != 0 {
            return Err(ErrorMessage::ErrorInDeserialization(format!("Payload in verack message has to be 0: {:?}", payload_size)));
        }
        
        let receive_checksum = <[u8; CHECKSUM_SIZE] as Deserializable>::deserialize(&mut buffer)?;
        if !VERACK_CHECKSUM.eq(&receive_checksum) {
            return Err(ErrorMessage::ErrorInDeserialization(format!("Checksum isn't the same: {:?} != {:?}", VERACK_CHECKSUM, receive_checksum)));
        }

        Ok(VerackMessage::new(magic_bytes))
    }

}

#[cfg(test)]
mod tests {
    use crate::messages::{
        serializable::Serializable, 
        deserializable::Deserializable,
        error_message::ErrorMessage,
    };

    use super::{
        VerackMessage,
        VERACK_TYPE,
        VERACK_CHECKSUM,
    };

    #[test]
    fn test01_serialize() -> Result<(), ErrorMessage>{
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];
        let verack_message = VerackMessage::new(magic_bytes);
        let mut stream: Vec<u8> = Vec::new();

        verack_message.serialize(&mut stream)?;
    
        let mut expected_stream: Vec<u8> = Vec::new();
        magic_bytes.serialize(&mut expected_stream)?;
        VERACK_TYPE.serialize(&mut expected_stream)?;
        vec![0, 0, 0, 0].serialize(&mut expected_stream)?;
        VERACK_CHECKSUM.serialize(&mut expected_stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize() -> Result<(), ErrorMessage> {
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];
        
        let mut stream: Vec<u8> = Vec::new();
        magic_bytes.serialize(&mut stream)?;
        VERACK_TYPE.serialize(&mut stream)?;
        vec![0, 0, 0, 0].serialize(&mut stream)?;
        VERACK_CHECKSUM.serialize(&mut stream)?;
        let mut stream: &[u8] = &stream;

        let expected_verack = VerackMessage::new(magic_bytes);

        let verack = VerackMessage::deserialize(&mut stream)?;

        assert_eq!(expected_verack, verack);

        Ok(())
    }

}