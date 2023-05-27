use k256::ecdsa::VerifyingKey;

use super::error_wallet::ErrorWallet;

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

    