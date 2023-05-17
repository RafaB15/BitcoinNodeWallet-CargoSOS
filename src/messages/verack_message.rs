use super::{
    message::Message,
    command_name::CommandName,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable, 
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
        message: &Self,
    ) -> Result<[u8; 4], ErrorSerialization> {

        Ok(VERACK_CHECKSUM)
    }
}

impl Serializable for VerackMessage {

    fn serialize(&self, _: &mut dyn Write) -> Result<(), ErrorSerialization> {
        Ok(())
    }
}

impl Deserializable for VerackMessage {
    fn deserialize(_: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(VerackMessage)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Serializable,
        Deserializable,
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
        magic_bytes.serialize(&mut expected_stream)?;
        CommandName::Verack.serialize(&mut expected_stream)?;
        vec![0, 0, 0, 0].serialize(&mut expected_stream)?;
        VERACK_CHECKSUM.serialize(&mut expected_stream)?;

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
        magic_bytes.serialize(&mut stream)?;
        CommandName::Verack.serialize(&mut stream)?;
        vec![0, 0, 0, 0].serialize(&mut stream)?;
        VERACK_CHECKSUM.serialize(&mut stream)?;
        let mut stream: &[u8] = &stream;

        let expected_verack = VerackMessage::deserialize_message(&mut stream, header)?;

        let verack = VerackMessage::deserialize(&mut stream)?;

        assert_eq!(expected_verack, verack);

        Ok(())
    }
}
