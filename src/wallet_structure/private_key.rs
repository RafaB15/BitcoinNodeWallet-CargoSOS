use k256::ecdsa::SigningKey;
use bs58::decode;
use std::convert::TryInto;

use super::error_wallet::ErrorWallet;

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateKey {
    key: SigningKey,
}

impl PrivateKey {

    /// Recibe un string que representa una llave privada en formato WIF (no comprimida)
    /// Devuelve un objeto PrivateKey
    pub fn new(string_key: &str) -> Result<PrivateKey, ErrorWallet> {
        let decoded_string = match decode(string_key).into_vec() {
            Ok(decoded_string) => decoded_string,
            Err(_) => return Err(ErrorWallet::CannotGeneratePrivateKey("Cannot decode private key".to_string())),
        };

        let key = match SigningKey::from_slice(&decoded_string[1..33]) {
            Ok(key) => key,
            Err(e) => return Err(ErrorWallet::CannotGeneratePrivateKey(format!("Cannot generate SigningKey object from {:?}, error : {:?}",decoded_string, e))),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_key_creation() {
        let private_key_string = "91fTpaUpEJPCgVUJFePpch3XDvP3qs8fVC395DMThYxNBNbusmx".to_string();
        let private_key = PrivateKey::new(&private_key_string).unwrap();
        let bytes: [u8; 32] = [0x0a, 0x52, 0x65, 0x08, 0x2e, 0x24, 0x11, 0x5f, 0x77, 0x54, 0x0a, 0xb3, 0xb8, 0xc2, 0xb9, 0x20, 0x60, 0xaa, 0x30, 0xd6, 0xd2, 0xb8, 0x1a, 0x08, 0x5d, 0x71, 0xab, 0x37, 0xed, 0xa7, 0x68, 0x91];
        let signing_bytes = private_key.as_bytes().unwrap();
        assert!(signing_bytes == bytes);
    }
}