use super::error_wallet::ErrorWallet;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
};

use crate::block_structure::hash::hash160;

use std::{
    convert::TryFrom,
    io::{Read, Write},
};

pub const PUBLIC_KEY_SIZE: usize = 33;
pub type PublicKeyType = [u8; PUBLIC_KEY_SIZE];

/// It's the internal representation of a public key for an account
#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey {
    key: PublicKeyType,
}

impl PublicKey {
    pub fn new(public_key_bytes: &PublicKeyType) -> PublicKey {
        PublicKey {
            key: *public_key_bytes,
        }
    }

    /// Returns the public key as a byte array
    pub fn as_bytes(&self) -> PublicKeyType {
        self.key
    }

    pub fn get_hashed_160(&self) -> Result<[u8; 20], ErrorSerialization> {
        hash160(&self.key)
    }
}

impl SerializableInternalOrder for PublicKey {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.key.io_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for PublicKey {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let bytes = <[u8; 33]>::io_deserialize(stream)?;
        let public_key = PublicKey::new(&bytes);
        Ok(public_key)
    }
}

impl TryFrom<String> for PublicKey {
    type Error = ErrorWallet;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut bytes: Vec<u8> = Vec::new();

        for (i, char) in value.chars().enumerate().step_by(2) {
            let mut byte = String::new();
            byte.push(char);

            match value.chars().nth(i + 1) {
                Some(next_char) => byte.push(next_char),
                None => byte.push('0'),
            }

            match u8::from_str_radix(&byte, 16) {
                Ok(byte) => bytes.push(byte),
                Err(e) => {
                    return Err(ErrorWallet::CannotGeneratePublicKey(format!(
                        "Error while converting a string ({byte}) into hexa: {:?}",
                        e
                    )))
                }
            }
        }

        let bytes: PublicKeyType = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(bytes) => {
                return Err(ErrorWallet::CannotGeneratePublicKey(format!(
                    "Cannot convert string to bytes, we get: {:?}",
                    bytes
                )))
            }
        };

        Ok(PublicKey::new(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_public_ket_creation() {
        let pubkey_bytes: [u8; 33] = [
            0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
            0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
            0x35, 0x14, 0x92, 0x4A, 0x22,
        ];
        let public_key = PublicKey::new(&pubkey_bytes);

        assert_eq!(public_key.as_bytes(), pubkey_bytes);
    }

    #[test]
    fn test_02_correct_public_key_serialization() {
        let pubkey_bytes: [u8; 33] = [
            0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
            0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
            0x35, 0x14, 0x92, 0x4A, 0x22,
        ];
        let public_key = PublicKey::new(&pubkey_bytes);
        let mut serialized_public_key = Vec::new();
        public_key.io_serialize(&mut serialized_public_key).unwrap();
        assert_eq!(serialized_public_key, pubkey_bytes);
    }

    #[test]
    fn test_03_correct_public_key_deserialization() {
        let pubkey_bytes: [u8; 33] = [
            0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
            0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
            0x35, 0x14, 0x92, 0x4A, 0x22,
        ];
        let public_key = PublicKey::new(&pubkey_bytes);
        let mut serialized_public_key = Vec::new();
        public_key.io_serialize(&mut serialized_public_key).unwrap();
        let deserialized_pubkey =
            PublicKey::io_deserialize(&mut serialized_public_key.as_slice()).unwrap();
        assert_eq!(deserialized_pubkey, public_key);
    }

    #[test]
    fn test_04_correct_hashing_160() {
        let pubkey_bytes: [u8; 33] = [
            0x02, 0x50, 0x86, 0x3a, 0xd6, 0x4a, 0x87, 0xae, 0x8a, 0x2f, 0xe8, 0x3c, 0x1a, 0xf1,
            0xa8, 0x40, 0x3c, 0xb5, 0x3f, 0x53, 0xe4, 0x86, 0xd8, 0x51, 0x1d, 0xad, 0x8a, 0x04,
            0x88, 0x7e, 0x5b, 0x23, 0x52,
        ];
        let public_key = PublicKey::new(&pubkey_bytes);
        let hashed_160 = public_key.get_hashed_160().unwrap();

        let hash_expected = [
            0xF5, 0x4A, 0x58, 0x51, 0xE9, 0x37, 0x2B, 0x87, 0x81, 0x0A, 0x8E, 0x60, 0xCD, 0xD2,
            0xE7, 0xCF, 0xD8, 0x0B, 0x6E, 0x31,
        ];

        assert_eq!(hashed_160, hash_expected);
    }
}
