use super::{
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian, 
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read, 
    Write
};

pub const VERACK_CHECKSUM: [u8; 4] = [0x5d, 0xf6, 0xe0, 0xe2];

#[derive(Debug, std::cmp::PartialEq)]
pub struct VerackMessage;

impl Message for VerackMessage {

    fn get_command_name() -> CommandName {
        CommandName::Verack
    }

    fn calculate_checksum(
        _: &[u8],
    ) -> Result<[u8; 4], ErrorSerialization> {

        Ok(VERACK_CHECKSUM)
    }
}

impl SerializableLittleEndian for VerackMessage {

    fn le_serialize(&self, _: &mut dyn Write) -> Result<(), ErrorSerialization> {
        Ok(())
    }
}

impl DeserializableLittleEndian for VerackMessage {
    fn le_deserialize(_: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(VerackMessage)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SerializableLittleEndian,
        DeserializableLittleEndian,
        ErrorSerialization,
        VerackMessage,
        VERACK_CHECKSUM,
    };

    use crate::messages::{
        message::Message,
        message_header::MessageHeader,
        command_name::CommandName,
    };

    #[test]
    fn test01_serialize() -> Result<(), ErrorSerialization>{
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];

        let verack_message = VerackMessage;
        let mut stream: Vec<u8> = Vec::new();
      
        VerackMessage::serialize_message(
            &mut stream,
            magic_bytes, 
            &verack_message,
        )?;
        let mut expected_stream: Vec<u8> = Vec::new();
        magic_bytes.le_serialize(&mut expected_stream)?;
        CommandName::Verack.le_serialize(&mut expected_stream)?;
        vec![0, 0, 0, 0].le_serialize(&mut expected_stream)?;
        VERACK_CHECKSUM.le_serialize(&mut expected_stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize() -> Result<(), ErrorSerialization> {
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];

        let header = MessageHeader {
            magic_numbers: magic_bytes,
            command_name: CommandName::Verack,
            payload_size: 0,
            checksum: VERACK_CHECKSUM,
        };
      
        let mut stream: Vec<u8> = Vec::new();
        magic_bytes.le_serialize(&mut stream)?;
        CommandName::Verack.le_serialize(&mut stream)?;
        vec![0, 0, 0, 0].le_serialize(&mut stream)?;
        VERACK_CHECKSUM.le_serialize(&mut stream)?;
        let mut stream: &[u8] = &stream;

        let expected_verack = VerackMessage::deserialize_message(&mut stream, header)?;

        let verack = VerackMessage::le_deserialize(&mut stream)?;

        assert_eq!(expected_verack, verack);

        Ok(())
    }
}
