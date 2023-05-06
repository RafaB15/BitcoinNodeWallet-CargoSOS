use std::io::Write;
use super::error_message::ErrorMessage;

pub trait Serializable {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage>;
    
}