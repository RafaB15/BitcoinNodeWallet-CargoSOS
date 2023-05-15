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

pub const SEND_HEADERS_CHECKSUM: [u8; 4] = [0x5d, 0xf6, 0xe0, 0xe2];

#[derive(Debug, std::cmp::PartialEq)]
pub struct SendHeadersMessage;

impl SendHeadersMessage {
  
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

        let message = SendHeadersMessage::deserialize(&mut buffer)?;

        if message_header.payload_size != 0 {
            return Err(ErrorSerialization::ErrorInDeserialization(format!("Payload in send headers message has to be 0: {:?}", message_header.payload_size)));
        }
        
        if !SEND_HEADERS_CHECKSUM.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(format!("Checksum isn't the same: {:?} != {:?}", SEND_HEADERS_CHECKSUM, message_header.checksum)));
        }

        Ok(message)
    }
}

impl Serializable for SendHeadersMessage {

    fn serialize(&self, _: &mut dyn Write) -> Result<(), ErrorSerialization> {
        Ok(())
    }
}

impl Deserializable for SendHeadersMessage {
    fn deserialize(_: &mut dyn Read) -> Result<Self, ErrorSerialization> {        
        Ok(SendHeadersMessage)
    }
}