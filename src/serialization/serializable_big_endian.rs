use super::error_serialization::ErrorSerialization;
use std::io::Write;
use std::net::Ipv6Addr;

pub trait SerializableBigEndian {
    fn serialize_big_endian(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization>;
}

impl SerializableBigEndian for u16 {
    fn serialize_big_endian(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.to_be_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing u16 in Big endian".to_string(),
            )),
        }
    }
}

impl SerializableBigEndian for Ipv6Addr {
    fn serialize_big_endian(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match stream.write(&self.octets()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing Ipv6Addr in Big endian".to_string(),
            )),
        }
    }
}

impl SerializableBigEndian for [u8] {
    fn serialize_big_endian(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        match stream.write(self) {
            Ok(_) => Ok(()),
            _ => Err(ErrorSerialization::ErrorInSerialization(
                "Serializing Vec<u8>".to_string(),
            )),
        }
    }
}


#[cfg(test)]
mod tests {

    use super::{ErrorSerialization, SerializableBigEndian};

    use std::net::Ipv6Addr;

    #[test]
    fn test01_serialize_correctly_u16() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x3F, 0x9E];

        let mut stream: Vec<u8> = Vec::new();
        let number: u16 = 16286;

        number.serialize_big_endian(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_serialize_correctly_ipv6() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xc0, 0x0a,
            0x02, 0xff,
        ];

        let mut stream: Vec<u8> = Vec::new();

        let ip: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x02ff);

        ip.serialize_big_endian(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }
}
