use super::{
    message_header::MessageHeader,
};

use crate::serialization::{
    deserializable::Deserializable, error_serialization::ErrorSerialization,
    serializable::Serializable,
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
        let mut buffer: Vec<u8> = vec![0; message_header.payload_size as usize];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorWhileReading);
        }
        let mut buffer: &[u8] = &buffer[..];

        let message = VerackMessage::deserialize(&mut buffer)?;

        if message_header.payload_size != 0 {
            return Err(ErrorSerialization::ErrorInDeserialization(format!("Payload in verack message has to be 0: {:?}", message_header.payload_size)));
        }
        
        if !VERACK_CHECKSUM.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(format!("Checksum in verack isn't the same: {:?} != {:?}", VERACK_CHECKSUM, message_header.checksum)));
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
        MessageHeader,
        VerackMessage,
        VERACK_CHECKSUM,
    };

    use crate::messages::{
        message,
        command_name::CommandName,
    };

    #[test]
    fn test01_serialize() -> Result<(), ErrorSerialization>{
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];

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
