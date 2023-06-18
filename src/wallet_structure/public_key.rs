use super::error_wallet::ErrorWallet;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
};

use std::{
    convert::TryFrom,
    io::{Read, Write},
};

pub const PUBLIC_KEY_SIZE: usize = 33;
pub type PublicKeyType = [u8; PUBLIC_KEY_SIZE];

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey {
    key: PublicKeyType,
}

impl PublicKey {
    pub fn new(public_key_bytes: &PublicKeyType) -> PublicKey {
        PublicKey {
            key: public_key_bytes.clone(),
        }
    }

    pub fn as_bytes(&self) -> PublicKeyType {
        self.key.clone()
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
