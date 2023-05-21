use crate::serialization::{
    error_serialization::ErrorSerialization,
};

use bitcoin_hashes::{sha256, sha256d, Hash};

pub const HASH_TYPE_SIZE: usize = 32;
pub const HASH_TYPE_REDUCE_SIZE: usize = 4;

pub type HashType = [u8; HASH_TYPE_SIZE];
pub type HashTypeReduced = [u8; HASH_TYPE_REDUCE_SIZE];

pub fn hash256(bytes: &[u8]) -> Result<HashType, ErrorSerialization> {
    let hash_bytes = sha256::Hash::hash(bytes);
    let hash_bytes: &[u8] = hash_bytes.as_ref();
    let hash_bytes_32: HashType = match hash_bytes.try_into() {
        Ok(hash_bytes_32) => hash_bytes_32,
        _ => {
            return Err(ErrorSerialization::ErrorInSerialization(
                "While hashing".to_string(),
            ))
        }
    };

    Ok(hash_bytes_32)
}

pub fn hash256d(bytes: &[u8]) -> Result<HashType, ErrorSerialization> {
    let hash_bytes = sha256d::Hash::hash(bytes);
    let hash_bytes: &[u8] = hash_bytes.as_ref();
    let hash_bytes_32: HashType = match hash_bytes.try_into() {
        Ok(hash_bytes_32) => hash_bytes_32,
        _ => {
            return Err(ErrorSerialization::ErrorInSerialization(
                "While hashing".to_string(),
            ))
        }
    };

    Ok(hash_bytes_32)
}

pub fn hash256d_reduce(bytes: &[u8]) -> Result<HashTypeReduced, ErrorSerialization> {
    let hash_byte_32: HashType = hash256d(bytes)?;

    let hash_byte_4: HashTypeReduced = match hash_byte_32[..4].try_into() {
        Ok(hash_byte_4) => hash_byte_4,
        _ => {
            return Err(ErrorSerialization::ErrorInSerialization(
                "While reduce hashing".to_string(),
            ))
        }
    };

    Ok(hash_byte_4)
}