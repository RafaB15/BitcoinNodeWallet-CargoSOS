use super::{
    serializable::Serializable,
    deserializable::Deserializable,
};

pub trait Payload : Serializable + Deserializable {
    
    fn get_message_type(&self) -> [u8; 12];
}