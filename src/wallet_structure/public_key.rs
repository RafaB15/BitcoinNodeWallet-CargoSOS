use super::error_wallet::ErrorWallet;

use k256::ecdsa::VerifyingKey;
use std::convert::TryFrom;

pub const PUBLIC_KEY_SIZE: usize = 33;
pub type PublicKeyType = [u8; PUBLIC_KEY_SIZE];

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey {
    key: VerifyingKey,
    compressed_key : PublicKeyType,
}

impl PublicKey {
    pub fn new (public_key_bytes: &PublicKeyType) -> Result<PublicKey, ErrorWallet> {
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

impl TryFrom<String> for PublicKey {
    type Error = ErrorWallet;

    fn try_from(value: String) -> Result<Self, Self::Error> {

        let mut bytes: Vec<u8> = Vec::new();

        for (i, char) in value.chars().enumerate().step_by(2) {
            let mut byte = String::new();
            byte.push(char);
            
            match value.chars().nth(i+1) {
                Some(next_char) => byte.push(next_char),
                None => byte.push('0'),
            }

            match u8::from_str_radix(&byte, 16) {
                Ok(byte) => bytes.push(byte),
                Err(e) => return Err(ErrorWallet::CannotGeneratePublicKey(format!("Error while converting a string ({byte}) into hexa: {:?}", e))),
            }
        }

        let bytes: PublicKeyType = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(bytes) => return Err(ErrorWallet::CannotGeneratePublicKey(format!("Cannot convert string to bytes, we get: {:?}", bytes))),
        };

        PublicKey::new(&bytes)
    }
}


    