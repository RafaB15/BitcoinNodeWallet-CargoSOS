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
        match stream.write(&self.octets()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

#[cfg(test)]
mod tests {
    
    use super::{
        SerializableBigEndian,
        ErrorMessage,
    };

    use std::net::Ipv6Addr;

    #[test]
    fn test01_serialize_correctly_u16() -> Result<(), ErrorMessage> {
        let stream_esperado: Vec<u8> = vec![0x3F, 0x9E];
        
        let mut stream: Vec<u8> = Vec::new();
        let numero: u16 = 16286;

        numero.serialize_big_endian(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

    #[test]
    fn test02_serialize_correctly_ipv6() -> Result<(), ErrorMessage> {
        let stream_esperado: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xc0, 0x0a, 0x02, 0xff];
        
        let mut stream: Vec<u8> = Vec::new();
        
        let ip: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);

        ip.serialize_big_endian(&mut stream)?;

        assert_eq!(stream_esperado, stream);

        Ok(())
    }

}