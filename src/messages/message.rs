use crate::messages::{
    verack_message::{VerackMessage, VERACK_TYPE}, 
    version_message::{VersionMessage, VERSION_TYPE}
};

use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};


pub struct Message<Payload>
    where Payload : Deserializable + Serializable
{
    pub magic_bytes: [u8; 4],
    pub payload: Payload,
}

impl<Payload> Message<Payload> 
    where Payload : Deserializable + Serializable
{
    pub fn new(magic_bytes: [u8; 4], payload: Payload) -> Self {
        Message { 
            magic_bytes, 
            payload,
        }
    }
}

impl<Payload> Serializable for Message<Payload>
    where Payload : Deserializable + Serializable
{
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {

        if stream.write(&self.magic_bytes).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        };

        self.payload.serialize(stream)
    }
}

impl<Payload> Deserializable for Message<Payload> 
    where Payload : Deserializable + Serializable
{
    type Value = Self;
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        /* Tener en cuenta
        pub message_type: [u8; 12],
        pub payload_size: u32,
        pub checksum: [u8; 4],
         */

        let mut magic_buff = [0u8; 4];
        let mut message_type = [0u8; 12];

        if stream.read_exact(&mut magic_buff).is_err(){
            return Err(ErrorMessage::ErrorInDeserialization);
        }

        if stream.read_exact(&mut message_type).is_err(){
            return Err(ErrorMessage::ErrorInDeserialization);
        }    

        let payload: Payload = match message_type {
            VERACK_TYPE => VerackMessage::deserialize(stream)?,
            VERSION_TYPE => VersionMessage::deserialize(stream)?,
            _ => return Err(ErrorMessage::ErrorMessageUnknown),
        };

        todo!()
    }
}