use crate::messages::{verack_message::VerackMessage, version_message::{VersionMessage, self}};

use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    payload::Payload,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};


pub struct Message<Load>
    where Load : Payload
{
    pub magic_bytes: [u8; 4],
    pub payload: Load,
}

impl<Load> Message<Load> 
    where Load : Payload
{
    pub fn new(magic_bytes: [u8; 4], payload: Load) -> Self {
        Message { 
            magic_bytes, 
            payload,
        }
    }
}

impl<Load> Serializable for Message<Load>
    where Load : Payload
{
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {

        if stream.write(&self.magic_bytes).is_err() {
            return Err(ErrorMessage::ErrorInSerialization);
        };

        self.payload.serialize(stream)
    }
}

impl<Load> Deserializable for Message<Load> 
    where Load : Payload
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

        let verack_type = VerackMessage::get_message_type();
        let version_type = VersionMessage::get_message_type();

        let payload = match message_type {
            verack_type => VerackMessage::deserialize(stream)?,
            version_message => VersionMessage::deserialize(stream)?,
            _ => return Err(ErrorMessage::ErrorMessageUnknown),
        };

        todo!()
    }
}