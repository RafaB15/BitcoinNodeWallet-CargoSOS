use crate::serialization::error_serialization::ErrorSerialization;

use bitcoin_hashes::{hash160, sha256, sha256d, Hash};

pub const HASH_TYPE_SIZE: usize = 32;
pub const HASH_TYPE_REDUCE_SIZE: usize = 4;

pub type HashType = [u8; HASH_TYPE_SIZE];
pub type HashTypeReduced = [u8; HASH_TYPE_REDUCE_SIZE];

/// It hashes a byte array using sha256
///
/// ### Error
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when there is an error in the serialization
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

/// It hashes to times a byte array using sha256
///
/// ### Error
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when there is an error in the serialization
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

/// It hashes a byte array using hash160
///
/// ### Error
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when there is an error in the serialization
pub fn hash160(bytes: &[u8]) -> Result<[u8; 20], ErrorSerialization> {
    let hash_bytes = hash160::Hash::hash(bytes);
    let hash_bytes: &[u8] = hash_bytes.as_ref();
    let hash_bytes_20: [u8; 20] = match hash_bytes.try_into() {
        Ok(hash_bytes_20) => hash_bytes_20,
        _ => {
            return Err(ErrorSerialization::ErrorInSerialization(
                "While hashing".to_string(),
            ))
        }
    };

    Ok(hash_bytes_20)
}

/// It hashes to times a byte array using sha256 and then it reduces it to 4 bytes
///
/// ### Error
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when there is an error in the serialization
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash256() {
        let bytes = vec![
            0x02, 0x50, 0x86, 0x3a, 0xd6, 0x4a, 0x87, 0xae, 0x8a, 0x2f, 0xe8, 0x3c, 0x1a, 0xf1,
            0xa8, 0x40, 0x3c, 0xb5, 0x3f, 0x53, 0xe4, 0x86, 0xd8, 0x51, 0x1d, 0xad, 0x8a, 0x04,
            0x88, 0x7e, 0x5b, 0x23, 0x52,
        ];
        let hash = hash256(&bytes).unwrap();
        let hash_expected = [
            0x0b, 0x7c, 0x28, 0xc9, 0xb7, 0x29, 0x0c, 0x98, 0xd7, 0x43, 0x8e, 0x70, 0xb3, 0xd3,
            0xf7, 0xc8, 0x48, 0xfb, 0xd7, 0xd1, 0xdc, 0x19, 0x4f, 0xf8, 0x3f, 0x4f, 0x7c, 0xc9,
            0xb1, 0x37, 0x8e, 0x98,
        ];
        assert_eq!(hash, hash_expected);
    }

    #[test]
    fn test_hash160() {
        let bytes = vec![
            0x02, 0x50, 0x86, 0x3a, 0xd6, 0x4a, 0x87, 0xae, 0x8a, 0x2f, 0xe8, 0x3c, 0x1a, 0xf1,
            0xa8, 0x40, 0x3c, 0xb5, 0x3f, 0x53, 0xe4, 0x86, 0xd8, 0x51, 0x1d, 0xad, 0x8a, 0x04,
            0x88, 0x7e, 0x5b, 0x23, 0x52,
        ];
        let hash = hash160(&bytes).unwrap();
        let hash_expected = [
            0xF5, 0x4A, 0x58, 0x51, 0xE9, 0x37, 0x2B, 0x87, 0x81, 0x0A, 0x8E, 0x60, 0xCD, 0xD2,
            0xE7, 0xCF, 0xD8, 0x0B, 0x6E, 0x31,
        ];
        assert_eq!(hash, hash_expected);
    }

    #[test]
    fn test_hash256d() {
        let bytes = vec![
            0x02, 0x50, 0x86, 0x3a, 0xd6, 0x4a, 0x87, 0xae, 0x8a, 0x2f, 0xe8, 0x3c, 0x1a, 0xf1,
            0xa8, 0x40, 0x3c, 0xb5, 0x3f, 0x53, 0xe4, 0x86, 0xd8, 0x51, 0x1d, 0xad, 0x8a, 0x04,
            0x88, 0x7e, 0x5b, 0x23, 0x52,
        ];
        let hash = hash256d(&bytes).unwrap();
        let hash_expected = [
            0xF9, 0x6F, 0x26, 0xC8, 0x7C, 0xF6, 0x35, 0xE0, 0x50, 0x35, 0x96, 0x96, 0x7E, 0xE2,
            0x5C, 0xD3, 0x2D, 0xCE, 0x2A, 0x67, 0x84, 0x27, 0x9F, 0x30, 0x21, 0x21, 0x46, 0x8A,
            0xD3, 0x52, 0x0F, 0x94,
        ];
        assert_eq!(hash, hash_expected);
    }

    #[test]
    fn test_04_correct_hash256d_reduce() {
        let bytes = vec![
            0x00, 0xF5, 0x4A, 0x58, 0x51, 0xE9, 0x37, 0x2B, 0x87, 0x81, 0x0A, 0x8E, 0x60, 0xCD,
            0xD2, 0xE7, 0xCF, 0xD8, 0x0B, 0x6E, 0x31,
        ];
        let hash = hash256d_reduce(&bytes).unwrap();
        let hash_expected = [0xC7, 0xF1, 0x8F, 0xE8];
        assert_eq!(hash, hash_expected);
    }
}
