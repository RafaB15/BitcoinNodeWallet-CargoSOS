use super::{
    message,
    message_header::MessageHeader,
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

impl VerackMessage {

    pub fn deserialize_message(
        stream: &mut dyn Read, 
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> 
    {
        let mut buffer: &[u8] = message::read_exact(stream, message_header.payload_size as usize)?;

        let message = VerackMessage::deserialize(&mut buffer)?;

        if message_header.payload_size != 0 {
            return Err(ErrorSerialization::ErrorInDeserialization(format!("Payload in verack message has to be 0: {:?}", message_header.payload_size)));
        }
        
        if !VERACK_CHECKSUM.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(format!("Checksum isn't the same: {:?} != {:?}", VERACK_CHECKSUM, message_header.checksum)));
        }

        Ok(message)
    }
}

impl Serializable for VerackMessage {

    fn serialize(&self, _: &mut dyn Write) -> Result<(), ErrorSerialization> {
        Ok(())
    }
}

impl Deserializable for VerackMessage {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(VerackMessage)
    }

}

#[cfg(test)]
mod tests {
    use super::{
        Serializable,
        Deserializable,
        ErrorSerialization,

        message,
        MessageHeader,
        VerackMessage,
        VERACK_CHECKSUM,
    };

    use crate::messages::command_name::CommandName;

    #[test]
    fn test01_serialize() -> Result<(), ErrorSerialization>{

        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];

        let header = MessageHeader {
            magic_numbers: magic_bytes,
            command_name: CommandName::Verack,
            payload_size: 0,
            checksum: VERACK_CHECKSUM,
        };

        let verack_message = VerackMessage;
        let mut stream: Vec<u8> = Vec::new();

        message::serialize_message(
            &mut stream,
            magic_bytes, 
            CommandName::Verack, 
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