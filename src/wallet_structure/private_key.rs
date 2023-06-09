use k256::ecdsa::SigningKey;
use std::convert::{TryFrom, TryInto};

use super::error_wallet::ErrorWallet;

pub const PRIVATE_KEY_SIZE: usize = 32;
pub type PrivateKeyType = [u8; PRIVATE_KEY_SIZE];

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateKey {
    key: SigningKey,
}

impl PrivateKey {

    /// Recibe un string que representa una llave privada en formato WIF (no comprimida)
    /// Devuelve un objeto PrivateKey
    pub fn new(private_key_bytes: &PrivateKeyType) -> Result<PrivateKey, ErrorWallet> {
        let key = match SigningKey::from_slice(private_key_bytes) {
            Ok(key) => key,
            Err(e) => return Err(ErrorWallet::CannotGeneratePrivateKey(format!("Cannot generate SigningKey object from {:?}, error : {:?}", private_key_bytes, e))),
        };
        
        Ok(PrivateKey {
            key,
        })
    }

    pub fn as_bytes(&self) -> Result<PrivateKeyType, ErrorWallet> {
        let bytes: PrivateKeyType = match self.key.to_bytes().try_into() {
            Ok(bytes) => bytes,
            Err(e) => return Err(ErrorWallet::CannotGeneratePrivateKey(format!("Cannot convert SigningKey object to bytes, error : {:?}", e))),
        };
        Ok(bytes)
    }
}

impl TryFrom<String> for PrivateKey {
    type Error = ErrorWallet;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut bytes: Vec<u8> = Vec::new();

        value.chars().enumerate().step_by(2).for_each(|(i, char)|{
            let mut byte = String::new();
            byte.push(char);
            if let Some(next_char) = value.chars().nth(i+1) {
                byte.push(next_char);
            } else {
                byte.push('0');
            }
            bytes.push(u8::from_str_radix(&byte, 16).unwrap());
        });

        let bytes: PrivateKeyType = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(e) => return Err(ErrorWallet::CannotGeneratePrivateKey(format!("Cannot convert string to bytes, error : {:?}", e))),
        };

        PrivateKey::new(&bytes)
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