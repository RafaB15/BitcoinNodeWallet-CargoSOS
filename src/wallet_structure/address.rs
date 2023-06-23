use super::error_wallet::ErrorWallet;

use crate::serialization::{
    deserializable_fix_size::DeserializableFixSize,
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::block_structure::transaction_output::TransactionOutput;

use std::{
    convert::TryInto,
    io::{Read, Write},
};

use bs58::decode;

pub const ADDRESS_SIZE: usize = 25;
pub type AddressType = [u8; ADDRESS_SIZE];

/// It's the internal representation of an address in an account
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    address_bytes: AddressType,
    address_string: String,
}

impl Address {
    /// Creates an address object from a string with a Bitcoin address
    ///
    /// ### Error
    ///  * `ErrorWallet::CannotDecodeAddress`: It will appear when address for an account cannot be generated
    pub fn new(address: &str) -> Result<Address, ErrorWallet> {
        if address.len() != 34 {
            return Err(ErrorWallet::CannotDecodeAddress(format!(
                "Invalid address length, expected 34, got {}",
                address.len()
            )));
        }
        let decoded_address = match decode(address).into_vec() {
            Ok(decoded_address) => decoded_address,
            Err(e) => {
                return Err(ErrorWallet::CannotDecodeAddress(format!(
                    "Cannot decode address {}, error : {:?}",
                    address, e
                )))
            }
        };
        let decoded_list: AddressType = match decoded_address.try_into() {
            Ok(decoded_list) => decoded_list,
            Err(e) => {
                return Err(ErrorWallet::CannotDecodeAddress(format!(
                    "Cannot convert decoded address to [u8; 25], error : {:?}",
                    e
                )))
            }
        };
        Ok(Address {
            address_bytes: decoded_list,
            address_string: address.to_string(),
        })
    }

    /// Extracts the hashed public key from the address
    fn extract_hashed_pk(&self) -> &[u8] {
        let hashed_pk = &self.address_bytes[1..21];
        hashed_pk
    }

    /// Generates the script pubkey for P2PKH from this address
    pub fn generate_script_pubkey_p2pkh(&self) -> Vec<u8> {
        let mut script_pubkey = vec![0x76, 0xa9, 0x14];
        script_pubkey.extend_from_slice(self.extract_hashed_pk());
        script_pubkey.extend_from_slice(&[0x88, 0xac]);
        script_pubkey
    }

    /// Returns true if the address owns the given transaction output (works for P2PKH) and false otherwise.
    pub fn verify_transaction_ownership(&self, txo: &TransactionOutput) -> bool {
        let pk_script = txo.pk_script.clone();
        if pk_script.len() != 25 {
            return false;
        }
        if pk_script[0] != 0x76
            || pk_script[1] != 0xa9
            || pk_script[2] != 0x14
            || pk_script[23] != 0x88
            || pk_script[24] != 0xac
        {
            return false;
        }
        let hashed_pk = &pk_script[3..23];
        hashed_pk == self.extract_hashed_pk()
    }
}

impl SerializableInternalOrder for Address {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        (self.address_string.len() as u64).le_serialize(stream)?;
        self.address_string.le_serialize(stream)?;
        self.address_bytes.io_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for Address {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let address_string_length = u64::le_deserialize(stream)? as usize;

        Ok(Address {
            address_string: String::deserialize_fix_size(stream, address_string_length)?,
            address_bytes: <[u8; 25] as DeserializableInternalOrder>::io_deserialize(stream)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_address_creation() {
        let address = "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs".to_string();
        let address_bytes = [
            0x00, 0xf5, 0x4a, 0x58, 0x51, 0xe9, 0x37, 0x2b, 0x87, 0x81, 0x0a, 0x8e, 0x60, 0xcd,
            0xd2, 0xe7, 0xcf, 0xd8, 0x0b, 0x6e, 0x31, 0xc7, 0xf1, 0x8f, 0xe8,
        ];
        let address = Address::new(&address).unwrap();
        assert!(address.address_string == "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs");
        assert!(address.address_bytes == address_bytes);
    }

    #[test]
    fn test_02_correct_extraction_of_hashed_pk() {
        let address = "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs".to_string();
        let hashed_pk: [u8; 20] = [
            0xf5, 0x4a, 0x58, 0x51, 0xe9, 0x37, 0x2b, 0x87, 0x81, 0x0a, 0x8e, 0x60, 0xcd, 0xd2,
            0xe7, 0xcf, 0xd8, 0x0b, 0x6e, 0x31,
        ];
        let address = Address::new(&address).unwrap();
        assert!(address.extract_hashed_pk() == hashed_pk);
    }
}
