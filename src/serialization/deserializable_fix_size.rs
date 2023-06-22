use super::error_serialization::ErrorSerialization;
use std::io::Read;

/// This trait is used to deserialize from a stream in Big endian while determining the size
pub trait DeserializableFixSize: Sized {
    fn deserialize_fix_size(
        stream: &mut dyn Read,
        sizes: usize,
    ) -> Result<Self, ErrorSerialization>;
}

impl DeserializableFixSize for String {
    fn deserialize_fix_size(
        stream: &mut dyn Read,
        sizes: usize,
    ) -> Result<Self, ErrorSerialization> {
        let mut buffer: Vec<u8> = vec![0; sizes];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorInDeserialization(
                "Deserializing string".to_string(),
            ));
        }

        match String::from_utf8(buffer.to_vec()) {
            Ok(string) => Ok(string),
            Err(_) => Err(ErrorSerialization::ErrorInDeserialization(format!(
                "Converting from utf8 to string: {:?}",
                buffer
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DeserializableFixSize, ErrorSerialization};

    #[test]
    fn test01_serialize_correctly_string() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![0x62, 0x75, 0x75];
        let mut stream: &[u8] = &stream;

        let expected_string: String = "buu".to_string();

        let string = String::deserialize_fix_size(&mut stream, expected_string.len() as usize)?;

        assert_eq!(expected_string, string);

        Ok(())
    }
}
