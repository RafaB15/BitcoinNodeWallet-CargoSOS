use super::error_message::ErrorMessage;
use std::net::Ipv6Addr;
use std::io::Read;

pub trait DeserializableBigEndian {
    type Value;

    fn deserialize_big_endian(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage>;
}

impl DeserializableBigEndian for u16 {
    type Value = u16;

    fn deserialize_big_endian(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 2];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(u16::from_be_bytes(buffer))
    }
}

impl DeserializableBigEndian for Ipv6Addr {
    type Value = Ipv6Addr;

    fn deserialize_big_endian(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage> {
        let mut buffer = [0u8; 16];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization);
        }
        Ok(Ipv6Addr::from(buffer))
    }
}