use crate::messages::payload::Payload;

use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};


pub struct Message
{
    pub magic_bytes: [u8; 4],
    pub payload: Payload,
}

impl Message {
    pub fn new(magic_bytes: [u8; 4], payload: Payload) -> Self {
        Message { 
            magic_bytes, 
            payload,
        }
    }
}

impl Serializable for Message {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {

        if stream.write(&self.magic_bytes).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        };

        self.payload.serialize(stream)
    }
}

impl Deserializable for Message {
    type Value = Self;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        
        let mut magic_buff = [0u8; 4];
        let mut message_type = [0u8; 12];

        if stream.read_exact(&mut magic_buff).is_err(){
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        if stream.read_exact(&mut message_type).is_err(){
            return Err(ErrorMessage::ErrorInDeserialization);
        }    

        let payload = Payload::get_from_message_type(message_type, stream)?;

        Ok(Message::new(magic_buff, payload))
    }
}