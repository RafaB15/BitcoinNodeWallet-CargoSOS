use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use std::io::{Read, Write};

pub const VERACK_TYPE: [u8; 12] = [118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0];

pub struct VerackMessage {}

impl VerackMessage {

    pub fn new() -> Self {
        VerackMessage {  }
    }
}

/* Tener en cuenta
pub payload_size: u32,
pub checksum: [u8; 4],
*/

impl Serializable for VerackMessage {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        todo!()
    }
}

impl Deserializable for VerackMessage {
    
    type Value = Self;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        todo!()
    }
}