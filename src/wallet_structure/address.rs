use bs58::decode;
use super::error_wallet::ErrorWallet;
use std::convert::TryInto;

pub struct Address {
    pub address_bytes: [u8; 25],
    pub address_string: String,
}

impl Address {
    /// Creates an address object from a string with a Bitcoin address
    pub fn new(address: &str) -> Result<Address, ErrorWallet> {
        if address.len() != 34 {
            return Err(ErrorWallet::InvalidAddress(format!("Invalid address length, expected 34, got {}", address.len())));
        }
        let decoded_address = match decode(address).into_vec() {
            Ok(decoded_address) => decoded_address,
            Err(e) => return Err(ErrorWallet::CannotDecodeAddress(format!("Cannot decode address {}, error : {:?}", address, e))),
        };
        let decoded_list: [u8; 25] = match decoded_address.try_into() {
            Ok(decoded_list) => decoded_list,
            Err(e) => return Err(ErrorWallet::CannotDecodeAddress(format!("Cannot convert decoded address to [u8; 25], error : {:?}", e))),
        };
        Ok(Address {
            address_bytes: decoded_list,
            address_string: address.to_string(),
        })
    }

    /// Extracts the hashed public key from the address
    pub fn extract_hashed_pk(&self) -> &[u8]{
        let hashed_pk = &self.address_bytes[1..21];
        hashed_pk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_address_creation() {
        let address = "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs".to_string();
        let address_bytes = [0x00, 0xf5, 0x4a, 0x58, 0x51, 0xe9, 0x37, 0x2b, 0x87, 0x81, 0x0a, 0x8e, 0x60, 0xcd, 0xd2, 0xe7, 0xcf, 0xd8, 0x0b, 0x6e, 0x31, 0xc7, 0xf1, 0x8f, 0xe8];
        let address = Address::new(&address).unwrap();
        assert!(address.address_string == "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs");
        assert!(address.address_bytes == address_bytes);
    }

    #[test]
    fn test_02_correct_extraction_of_hashed_pk() {
        let address = "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs".to_string();
        let hashed_pk: [u8; 20] = [0xf5, 0x4a, 0x58, 0x51, 0xe9, 0x37, 0x2b, 0x87, 0x81, 0x0a, 0x8e, 0x60, 0xcd, 0xd2, 0xe7, 0xcf, 0xd8, 0x0b, 0x6e, 0x31];
        let address = Address::new(&address).unwrap();
        assert!(address.extract_hashed_pk() == hashed_pk);
    }
}