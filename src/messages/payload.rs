use super::{
    serializable::Serializable,
    deserializable::Deserializable,
};

pub trait Payload : Serializable + Deserializable {
    
    fn get_message_type() -> [u8; 12];
}