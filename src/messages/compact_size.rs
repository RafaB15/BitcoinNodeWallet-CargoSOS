use crate::serialization::{
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_little_endian::SerializableLittleEndian,
};

use std::io::Write;

const MAX_U8: u64 = 0xFC;
const MAX_U16: u64 = 0xFFFF;
const MAX_U32: u64 = 0xFFFFFFFF;

const PREFIX_U16: u8 = 0xFD;
const PREFIX_U32: u8 = 0xFE;
const PREFIX_U64: u8 = 0xFF;

#[derive(Debug, std::cmp::PartialEq, Copy, Clone)]
pub struct CompactSize {
    pub value: u64,
}

impl CompactSize {
    pub fn new(value: u64) -> CompactSize {
        CompactSize { value }
    }
}

impl SerializableLittleEndian for CompactSize {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        if self.value <= MAX_U8 {
            (self.value as u8).le_serialize(stream)?;
        } else if self.value <= MAX_U16 {
            PREFIX_U16.le_serialize(stream)?;
            (self.value as u16).le_serialize(stream)?;
        } else if self.value <= MAX_U32 {
            PREFIX_U32.le_serialize(stream)?;
            (self.value as u32).le_serialize(stream)?;
        } else {
            PREFIX_U64.le_serialize(stream)?;
            self.value.le_serialize(stream)?;
        }

        Ok(())
    }
}

impl DeserializableLittleEndian for CompactSize {
    fn le_deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let mut buffer = [0u8; 1];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing CompactSize".to_string(),
            ));
        }

        match buffer {
            [PREFIX_U16] => Ok(CompactSize::new(u16::le_deserialize(stream)? as u64)),
            [PREFIX_U32] => Ok(CompactSize::new(u32::le_deserialize(stream)? as u64)),
            [PREFIX_U64] => Ok(CompactSize::new(u64::le_deserialize(stream)?)),
            [value] => Ok(CompactSize::new(value as u64)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{
        CompactSize, DeserializableLittleEndian, ErrorSerialization, SerializableLittleEndian,
    };

    #[test]
    fn test01_serialize_correctly_u8() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x54];

        let mut stream: Vec<u8> = Vec::new();
        let number: u8 = 84;
        let compact: CompactSize = CompactSize::new(number as u64);

        compact.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize_correctly_compact_size_of_u8() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![0x54];
        let mut stream: &[u8] = &stream;

        let expected_number: u8 = 84;
        let expected_compact: CompactSize = CompactSize::new(expected_number as u64);

        let compact = CompactSize::le_deserialize(&mut stream)?;

        assert_eq!(expected_compact, compact);

        Ok(())
    }

    #[test]
    fn test03_serialize_correctly_u16() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0xFD, 0x9E, 0x3F];

        let mut stream: Vec<u8> = Vec::new();
        let number: u16 = 16286;
        let compact: CompactSize = CompactSize::new(number as u64);

        compact.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test04_deserialize_correctly_compact_size_of_u16() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![0xFD, 0x9E, 0x3F];
        let mut stream: &[u8] = &stream;

        let expected_number: u16 = 16286;
        let expected_compact: CompactSize = CompactSize::new(expected_number as u64);

        let compact = CompactSize::le_deserialize(&mut stream)?;

        assert_eq!(expected_compact, compact);

        Ok(())
    }

    #[test]
    fn test05_serialize_correctly_u32() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0xFE, 0xAD, 0x83, 0xF8, 0x00];

        let mut stream: Vec<u8> = Vec::new();
        let number: u32 = 16_286_637;
        let compact: CompactSize = CompactSize::new(number as u64);

        compact.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test06_deserialize_correctly_compact_size_of_u32() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![0xFE, 0xAD, 0x83, 0xF8, 0x00];
        let mut stream: &[u8] = &stream;

        let expected_number: u32 = 16_286_637;
        let expected_compact: CompactSize = CompactSize::new(expected_number as u64);

        let compact = CompactSize::le_deserialize(&mut stream)?;

        assert_eq!(expected_compact, compact);

        Ok(())
    }

    #[test]
    fn test07_serialize_correctly_u64() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0xFF, 0xC7, 0x01, 0xBD, 0xDE, 0x19, 0x00, 0x00, 0x00];

        let mut stream: Vec<u8> = Vec::new();
        let number: u64 = 1111_1111_1111;
        let compact: CompactSize = CompactSize::new(number as u64);

        compact.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test08_deserialize_correctly_compact_size_of_u64() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![0xFF, 0xC7, 0x01, 0xBD, 0xDE, 0x19, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &stream;

        let expected_number: u64 = 1111_1111_1111;
        let expected_compact: CompactSize = CompactSize::new(expected_number as u64);

        let compact = CompactSize::le_deserialize(&mut stream)?;

        assert_eq!(expected_compact, compact);

        Ok(())
    }
}
