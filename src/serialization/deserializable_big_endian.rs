use super::error_serialization::ErrorSerialization;
use std::io::Read;
use std::net::Ipv6Addr;

pub trait DeserializableBigEndian: Sized {
    fn deserialize_big_endian(stream: &mut dyn Read) -> Result<Self, ErrorSerialization>;
}

impl DeserializableBigEndian for u16 {
    fn deserialize_big_endian(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 2];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing u16 in Big endian".to_string(),
            ));
        }
        Ok(u16::from_be_bytes(buffer))
    }
}

impl DeserializableBigEndian for Ipv6Addr {
    fn deserialize_big_endian(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 16];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing Ipv6Addr in Big endian".to_string(),
            ));
        }
        Ok(Ipv6Addr::from(buffer))
    }
}

impl DeserializableBigEndian for [u8; 32] {

    fn deserialize_big_endian(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 32];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing [u8; 32]".to_string()
            ));
        }
        let mut rev: Vec<u8> = Vec::new();
        
        for byte in buffer.iter().rev() {
            rev.push(*byte);
        }
        
        let buffer: [u8; 32] = match rev[0..32].try_into() {
            Ok(buffer) => buffer,
            _ => return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing [u8; 32]".to_string(),
            )),
        };
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {

    use super::{DeserializableBigEndian, ErrorSerialization};

    use std::net::Ipv6Addr;

    #[test]
    fn test01_deserialize_correctly_u16() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![0x3F, 0x9E];
        let mut stream: &[u8] = &stream;

        let expected_number: u16 = 16286;

        let number = u16::deserialize_big_endian(&mut stream)?;

        assert_eq!(expected_number, number);

        Ok(())
    }

    #[test]
    fn test02_deserialize_correctly_ipv6() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xc0, 0x0a,
            0x02, 0xff,
        ];
        let mut stream: &[u8] = &stream;

        let expected_ip: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);

        let ip = Ipv6Addr::deserialize_big_endian(&mut stream)?;

        assert_eq!(expected_ip, ip);

        Ok(())
    }
}
