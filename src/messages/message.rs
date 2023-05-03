use super::{
    serializable::Serializable,
    deserializable::Deserializable,
};

pub struct Message<Payload>
    where Payload : Serializable + Deserializable
{
    pub magic_bytes: [u8; 4],
    pub message_type: [u8; 12],
    pub payload_size: u32,
    pub checksum: [u8; 4],
    pub payload: Payload,
}

impl<Payload> Message<Payload> 
    where Payload : Serializable + Deserializable
{
    pub fn new(magic_bytes: [u8; 4], payload: Payload) {
        todo!()
    }
}

impl<Payload> Serializable for Message<Payload>
    where Payload : Serializable + Deserializable
{
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

impl<Payload> Deserializable for Message<Payload>
    where Payload : Serializable + Deserializable
{
    fn deserialize(data: Vec<u8>) -> Self {
        todo!()
    }
}