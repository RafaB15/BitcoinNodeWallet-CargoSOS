use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage, payload::Payload,
};

use std::io::{Read, Write};

pub const VERACK_TYPE: &[u8; 12] = b"verack\0\0\0\0\0\0";
pub const VERACK_CHECKSUM: [u8; 4] = [0x5d, 0xf6, 0xe0, 0xe2];

#[derive(Debug, std::cmp::PartialEq)]
pub struct VerackMessage {}

impl VerackMessage {

    pub fn new() -> Self {
        VerackMessage {  }
    }
}

impl VerackMessage {
    
    fn get_slice<const N: usize>(buffer: &[u8], inicio: usize) -> Result<[u8; N], ErrorMessage>{
        let slice: [u8; N] = match buffer[inicio..(N + inicio)].try_into() {
            Ok(slice) => slice,
            _ => return Err(ErrorMessage::ErrorInDeserialization),
        };

        Ok(slice)
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
        
        let mut num_buffer = [0u8; 8];
        if stream.read_exact(&mut num_buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        let payload_size = Self::get_slice::<4>(&num_buffer, 0)?;
        let payload_size = u32::from_be_bytes(payload_size);
        
        if payload_size != 0 {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        
        let receive_checksum = Self::get_slice::<4>(&num_buffer, 4)?;
        if !VERACK_CHECKSUM.eq(&receive_checksum) {
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        Ok(VerackMessage::new())
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
        let message_verack = VerackMessage::new();
        let mut stream: Vec<u8> = Vec::new();

        message_verack.serialize(&mut stream)?;
    
        let mut stream_esperado: Vec<u8> = Vec::new();
        stream_esperado.append(&mut VERACK_TYPE.to_vec());
        stream_esperado.append(&mut vec![0, 0, 0, 0]);
        stream_esperado.append(&mut VERACK_CHECKSUM.to_vec());

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test02_deserializar() -> Result<(), ErrorMessage> {
        let mut stream: Vec<u8> = Vec::new();
        stream.append(&mut vec![0, 0, 0, 0]);
        stream.append(&mut VERACK_CHECKSUM.to_vec());

        let mut stream: &[u8] = &stream[..];

        let verack_esperado = VerackMessage::new();

        let verack = VerackMessage::deserialize(&mut stream)?;

        assert_eq!(verack_esperado, verack);

        Ok(())
    }

}