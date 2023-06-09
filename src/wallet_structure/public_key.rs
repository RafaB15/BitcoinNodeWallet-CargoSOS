use k256::ecdsa::VerifyingKey;

use super::error_wallet::ErrorWallet;

use crate::serialization::{
    serializable_internal_order::SerializableInternalOrder,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};

use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey {
    key: VerifyingKey,
    compressed_key : [u8; 33],
}

impl PublicKey {
    pub fn new (public_key_bytes: &[u8; 33]) -> Result<PublicKey, ErrorWallet> {
        let key = match VerifyingKey::from_sec1_bytes(public_key_bytes) {
            Ok(verifying_key) => verifying_key,
            Err(e) => return Err(ErrorWallet::CannotGeneratePublicKey(format!("Cannot generate VerifyingKey object from {:?}, error : {:?}", public_key_bytes, e))),
        };
        Ok(PublicKey {
            key,
            compressed_key: public_key_bytes.clone(),
        })
    }
}

impl SerializableInternalOrder for PublicKey {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.compressed_key.io_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for PublicKey {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let bytes = <[u8; 33]>::io_deserialize(stream)?;
        let public_key = match PublicKey::new(&bytes) {
            Ok(public_key) => public_key,
            Err(e) => return Err(ErrorSerialization::ErrorInDeserialization(format!("Cannot deserialize public key, error : {:?}", e))),
        };

        Ok(public_key)
    }
}

    