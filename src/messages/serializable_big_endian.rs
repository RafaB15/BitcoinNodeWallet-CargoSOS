use super::error_message::ErrorMessage;
use std::net::Ipv6Addr;
use std::io::Write;

pub trait SerializableBigEndian {
    
    fn serialize_big_endian(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage>;
    
}

impl SerializableBigEndian for u16 {
    fn serialize_big_endian(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        match stream.write(&self.to_be_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl SerializableBigEndian for Ipv6Addr {
    fn serialize_big_endian(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        todo!()
        // .octets()
    }
}