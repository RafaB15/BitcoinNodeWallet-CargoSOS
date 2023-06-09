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

        let bytes: PublicKeyType = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(e) => return Err(ErrorWallet::CannotGeneratePublicKey(format!("Cannot convert string to bytes, error : {:?}", e))),
        };

        PublicKey::new(&bytes)
    }
}


    