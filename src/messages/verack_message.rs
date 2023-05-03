use super::{
    serializable::Serializable,
    deserializable::Deserializable,
};

use std::io::{Read, Write};

pub struct VerackMessage {}

impl VerackMessage {

    pub fn new() -> Self {
        VerackMessage {  }
    }
}

impl Serializable for VerackMessage {
    fn serialize(&self, stream: &mut dyn Write) {
        todo!()
    }
}

impl Deserializable for VerackMessage {
    fn deserialize(stream: &mut dyn Read) -> Self {
        todo!()
    }
}