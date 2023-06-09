use super::error_wallet::ErrorWallet;

use crate::serialization::{
    serializable_internal_order::SerializableInternalOrder,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};

use std::{
    io::{Read, Write},
    convert::TryInto,
};

use k256::ecdsa::SigningKey;

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateKey {
    key: SigningKey,
}

impl PrivateKey {

    /// Recibe un string que representa una llave privada en formato WIF (no comprimida)
    /// Devuelve un objeto PrivateKey
    pub fn new(private_key_bytes: &[u8; 32]) -> Result<PrivateKey, ErrorWallet> {
        let key = match SigningKey::from_slice(private_key_bytes) {
            Ok(key) => key,
            Err(e) => return Err(ErrorWallet::CannotGeneratePrivateKey(format!("Cannot generate SigningKey object from {:?}, error : {:?}", private_key_bytes, e))),
        };
        
        Ok(PrivateKey {
            key,
        })
    }

    pub fn as_bytes(&self) -> Result<[u8; 32], ErrorWallet> {
        let bytes: [u8; 32] = match self.key.to_bytes().try_into() {
            Ok(bytes) => bytes,
            Err(e) => return Err(ErrorWallet::CannotGeneratePrivateKey(format!("Cannot convert SigningKey object to bytes, error : {:?}", e))),
        };
        Ok(bytes)
    }

}

impl SerializableInternalOrder for PrivateKey {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        match self.as_bytes() {
            Ok(bytes) => bytes.io_serialize(stream)?,
            Err(e) => return Err(ErrorSerialization::ErrorInSerialization(format!("Cannot serialize private key, error : {:?}", e))),
        }

        Ok(())
    }
}

impl DeserializableInternalOrder for PrivateKey {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let bytes = <[u8; 32]>::io_deserialize(stream)?;
        let private_key = match PrivateKey::new(&bytes) {
            Ok(private_key) => private_key,
            Err(e) => return Err(ErrorSerialization::ErrorInDeserialization(format!("Cannot deserialize private key, error : {:?}", e))),
        };

        Ok(private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_key_creation() {
        let private_key_bytes: [u8; 32] = [0x0a, 0x52, 0x65, 0x08, 0x2e, 0x24, 0x11, 0x5f, 0x77, 0x54, 0x0a, 0xb3, 0xb8, 0xc2, 0xb9, 0x20, 0x60, 0xaa, 0x30, 0xd6, 0xd2, 0xb8, 0x1a, 0x08, 0x5d, 0x71, 0xab, 0x37, 0xed, 0xa7, 0x68, 0x91];
        let private_key = PrivateKey::new(&private_key_bytes).unwrap();
        let signing_bytes = private_key.as_bytes().unwrap();
        assert!(signing_bytes == private_key_bytes);
    }
}