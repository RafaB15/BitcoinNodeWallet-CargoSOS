use super::error_wallet::ErrorWallet;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
};

use std::{
    io::{Read, Write},
    str::FromStr,
};

use secp256k1::{
    SecretKey,
    Secp256k1,
};

pub const PRIVATE_KEY_SIZE: usize = 32;
pub type PrivateKeyType = [u8; PRIVATE_KEY_SIZE];

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateKey {
    key: SecretKey,
}

impl PrivateKey {
    /// Recibe un string que representa una llave privada en formato WIF (no comprimida)
    /// Devuelve un objeto PrivateKey
    pub fn new(private_key_bytes: &PrivateKeyType) -> Result<PrivateKey, ErrorWallet> {
        let key = match SecretKey::from_slice(private_key_bytes) {
            Ok(key) => key,
            Err(e) => {
                return Err(ErrorWallet::CannotGeneratePrivateKey(format!(
                    "Cannot generate PrivateKey object from {:?}, error : {:?}",
                    private_key_bytes, e
                )))
            }
        };

        Ok(PrivateKey { key })
    }

    pub fn as_bytes(&self) -> PrivateKeyType {
        self.key.secret_bytes()
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ErrorWallet> {
        println!("Signing message {:?} with lenght {}", message, message.len());
        let message = match secp256k1::Message::from_slice(message) {
            Ok(message) => message,
            Err(e) => {
                return Err(ErrorWallet::CannotSignMessage(format!(
                    "Cannot generate message to sign {:?}, error : {:?}",
                    message, e
                )))
            }
        };
        let secp = Secp256k1::new();
        Ok(secp.sign_ecdsa(&message, &self.key).serialize_der().to_vec())
    }
}

impl TryFrom<&str> for PrivateKey {
    type Error = ErrorWallet;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let private_key = match SecretKey::from_str(value) {
            Ok(private_key) => private_key,
            Err(e) => {
                return Err(ErrorWallet::CannotGeneratePrivateKey(format!(
                    "Cannot generate PrivateKey object from string, error : {:?}",
                    e
                )))
            }
        };

        Ok(PrivateKey {key: private_key})
    }
}

impl SerializableInternalOrder for PrivateKey {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.as_bytes().io_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for PrivateKey {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let bytes = <[u8; 32]>::io_deserialize(stream)?;
        let private_key = match PrivateKey::new(&bytes) {
            Ok(private_key) => private_key,
            Err(e) => {
                return Err(ErrorSerialization::ErrorInDeserialization(format!(
                    "Cannot deserialize private key, error : {:?}",
                    e
                )))
            }
        };

        Ok(private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_key_creation() {
        let private_key_bytes: [u8; 32] = [
            0x0a, 0x52, 0x65, 0x08, 0x2e, 0x24, 0x11, 0x5f, 0x77, 0x54, 0x0a, 0xb3, 0xb8, 0xc2,
            0xb9, 0x20, 0x60, 0xaa, 0x30, 0xd6, 0xd2, 0xb8, 0x1a, 0x08, 0x5d, 0x71, 0xab, 0x37,
            0xed, 0xa7, 0x68, 0x91,
        ];
        let private_key = PrivateKey::new(&private_key_bytes).unwrap();
        let signing_bytes = private_key.as_bytes();
        assert!(signing_bytes == private_key_bytes);
    }
}
