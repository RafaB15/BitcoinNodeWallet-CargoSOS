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
            payload 
        }
    }
}

impl<Load> Serializable for Message<Load>
    where Load : Payload
{
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        /* Tener en cuenta
        pub message_type: [u8; 12],
        pub payload_size: u32,
        pub checksum: [u8; 4],
         */
        todo!()
    }
}

impl<Load> Deserializable for Message<Load>
    
    where Load : Payload
{
    type Value = Self;
    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        todo!()
    }

    // std::cout << 4 << std::endl;
}