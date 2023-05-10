use super::{
    serializable::Serializable,
    deserializable::{
        Deserializable,
        get_slice,
    },
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
    
    type Value = Self;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        
        let mut position = 0;
        let mut buffer = [0u8; HEADER_SIZE];

        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        let magic_bytes = get_slice::<MAGIC_BYTES_SIZE>(&buffer, &mut position)?;
        
        let message_type = get_slice::<MASSAGE_TYPE_SIZE>(&buffer, &mut position)?;
        if !VERACK_TYPE.eq(&message_type) {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        
        let payload_size = get_slice::<PAYLOAD_SIZE>(&buffer, &mut position)?;        
        if u32::from_be_bytes(payload_size) != 0 {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        
        let receive_checksum = get_slice::<CHECKSUM_SIZE>(&buffer, &mut position)?;
        if !VERACK_CHECKSUM.eq(&receive_checksum) {
            return Err(ErrorMessage::ErrorInDeserialization);
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
    fn test01_serializar() -> Result<(), ErrorMessage>{
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];
        let message_verack = VerackMessage::new(magic_bytes);
        let mut stream: Vec<u8> = Vec::new();

        message_verack.serialize(&mut stream)?;
    
        let mut stream_esperado: Vec<u8> = Vec::new();
        stream_esperado.append(&mut magic_bytes.to_vec());
        stream_esperado.append(&mut VERACK_TYPE.to_vec());
        stream_esperado.append(&mut vec![0, 0, 0, 0]);
        stream_esperado.append(&mut VERACK_CHECKSUM.to_vec());

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test02_deserializar() -> Result<(), ErrorMessage> {
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];
        let mut stream: Vec<u8> = Vec::new();
        stream.append(&mut magic_bytes.to_vec());
        stream.append(&mut VERACK_TYPE.to_vec());
        stream.append(&mut vec![0, 0, 0, 0]);
        stream.append(&mut VERACK_CHECKSUM.to_vec());

        let mut stream: &[u8] = &stream[..];

        let verack_esperado = VerackMessage::new(magic_bytes);

        let verack = VerackMessage::deserialize(&mut stream)?;

        assert_eq!(verack_esperado, verack);

        Ok(())
    }

}