use super::{
    address::Address,
    error_wallet::ErrorWallet,
    private_key::{PrivateKey, PrivateKeyType},
    public_key::{PublicKey, PublicKeyType},
};

use crate::serialization::{
    deserializable_fix_size::DeserializableFixSize,
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::block_structure::{
    outpoint::Outpoint, transaction::Transaction, transaction_output::TransactionOutput,
    utxo_set::UTXOSet,
};

use std::{
    cmp::PartialEq,
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
};

/// It's the internal representation of an account in the wallet
#[derive(Debug, Clone)]
pub struct Account {
    pub account_name: String,
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: Address,
}

impl Account {
    pub fn new(
        name: &str,
        private_key_bytes: &PrivateKeyType,
        public_key_bytes: &PublicKeyType,
    ) -> Result<Account, ErrorWallet> {
        let account_name = name.to_string();
        let private_key = PrivateKey::new(private_key_bytes)?;
        let public_key = PublicKey::new(public_key_bytes);
        let address = Address::from_public_key(&public_key)?;

        Ok(Account {
            account_name,
            private_key,
            public_key,
            address,
        })
    }

    /// Returns true if the account owns the given transaction output (works for P2PKH) and false otherwise.
    pub fn verify_transaction_output_ownership(&self, txo: &TransactionOutput) -> bool {
        self.address.verify_transaction_ownership(txo)
    }

    /// Returns true if the account owns any transaction output given the transaction (works for P2PKH) and false otherwise.
    pub fn verify_transaction_ownership(&self, tx: &Transaction) -> bool {
        tx.verify_transaction_ownership(&self.address)
    }

    /// Returns the balance of the account in satoshis
    pub fn get_balance_in_satoshis(&self, utxo_set: UTXOSet) -> i64 {
        utxo_set.get_balance_in_satoshis(&self.address)
    }

    /// Returns the balance of the account in tbtc
    pub fn get_balance_in_tbtc(&self, utxo_set: UTXOSet) -> f64 {
        utxo_set.get_balance_in_tbtc(&self.address)
    }

    /// Returns a transaction given the amount and to whom it is sent
    ///
    /// ### Error
    ///  * `ErrorWallet::CannotCreateNewTransaction`: It will appear when a transaction cannot be created
    ///  * `ErrorWallet::NotEnoughFunds`: It will appear when an account does not have enough funds to create a transaction for the amount requested
    pub fn create_transaction_with_available_outputs(
        &self,
        to: Address,
        amount: i64,
        fee: i64,
        mut available_outputs: Vec<(Outpoint, TransactionOutput)>,
    ) -> Result<Transaction, ErrorWallet> {
        available_outputs.sort_by(|(_, a), (_, b)| b.value.cmp(&a.value));

        let mut input_amount = 0;
        let mut outputs_to_spend: Vec<(Outpoint, TransactionOutput)> = vec![];
        for (available_outpoint, available_transaction) in available_outputs.iter() {
            input_amount += available_transaction.value;
            outputs_to_spend.push((available_outpoint.clone(), available_transaction.clone()));
            if input_amount >= (amount + fee) {
                break;
            }
        }

        if input_amount < (amount + fee) {
            return Err(ErrorWallet::NotEnoughFunds(format!("Not enough funds to create the transaction. Input amount: {}. Output amount: {}. Fee: {}", input_amount, amount, fee)));
        }

        let outputs_to_spend: HashMap<Outpoint, TransactionOutput> =
            outputs_to_spend.into_iter().collect();

        match Transaction::from_account_to_address(&self, &outputs_to_spend, &to, amount, fee) {
            Ok(transaction) => Ok(transaction),
            Err(error) => Err(ErrorWallet::CannotCreateNewTransaction(format!(
                "Error while trying to create a new transaction. Error: {:?}",
                error
            ))),
        }
    }

    /// Returns a transaction given the amount and to whom it is sent
    ///
    /// ### Error
    ///  * `ErrorWallet::CannotCreateNewTransaction`: It will appear when a transaction cannot be created
    ///  * `ErrorWallet::NotEnoughFunds`: It will appear when an account does not have enough funds to create a transaction for the amount requested
    pub fn create_transaction(
        &self,
        to: Address,
        amount: i64,
        fee: i64,
        utxo_set: UTXOSet,
    ) -> Result<Transaction, ErrorWallet> {
        let available_outputs = utxo_set.get_utxo_list_with_outpoints(Some(&self.address));
        self.create_transaction_with_available_outputs(to, amount, fee, available_outputs)
    }

    /// Return a message signed with the private key of the account
    ///
    /// ### Error
    ///  * `ErrorWallet::CannotSignMessage`: It will appear when a transaction cannot be signed
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ErrorWallet> {
        self.private_key.sign(message)
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.account_name == other.account_name
    }
}

impl SerializableInternalOrder for Account {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        (self.account_name.len() as u64).le_serialize(stream)?;
        self.account_name.le_serialize(stream)?;

        self.private_key.io_serialize(stream)?;
        self.public_key.io_serialize(stream)?;
        self.address.io_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for Account {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let account_name_len = u64::le_deserialize(stream)? as usize;

        Ok(Account {
            account_name: String::deserialize_fix_size(stream, account_name_len)?,
            private_key: PrivateKey::io_deserialize(stream)?,
            public_key: PublicKey::io_deserialize(stream)?,
            address: Address::io_deserialize(stream)?,
        })
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Account Name: {}\n\twith address: {}",
            self.account_name, self.address
        )
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_1_correct_account_creation() {
        let priv_key_bytes: [u8; 32] = [
            0x0a, 0x52, 0x65, 0x08, 0x2e, 0x24, 0x11, 0x5f, 0x77, 0x54, 0x0a, 0xb3, 0xb8, 0xc2,
            0xb9, 0x20, 0x60, 0xaa, 0x30, 0xd6, 0xd2, 0xb8, 0x1a, 0x08, 0x5d, 0x71, 0xab, 0x37,
            0xed, 0xa7, 0x68, 0x91,
        ];
        let pubkey_bytes: [u8; 33] = [
            0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
            0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
            0x35, 0x14, 0x92, 0x4A, 0x22,
        ];
        let account = Account::new("test", &priv_key_bytes, &pubkey_bytes).unwrap();
        assert_eq!(account.account_name, "test");
        assert_eq!(account.public_key.as_bytes(), pubkey_bytes);
    }

    #[test]
    fn test_2_correct_account_serialization() {
        let priv_key_bytes: [u8; 32] = [
            0x0a, 0x52, 0x65, 0x08, 0x2e, 0x24, 0x11, 0x5f, 0x77, 0x54, 0x0a, 0xb3, 0xb8, 0xc2,
            0xb9, 0x20, 0x60, 0xaa, 0x30, 0xd6, 0xd2, 0xb8, 0x1a, 0x08, 0x5d, 0x71, 0xab, 0x37,
            0xed, 0xa7, 0x68, 0x91,
        ];
        let pubkey_bytes: [u8; 33] = [
            0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
            0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
            0x35, 0x14, 0x92, 0x4A, 0x22,
        ];

        let account_name = "test".to_string();
        let private_key = PrivateKey::new(&priv_key_bytes).unwrap();
        let public_key = PublicKey::new(&pubkey_bytes);
        let address = Address::from_public_key(&public_key).unwrap();

        let mut serialized_fields: Vec<u8> = Vec::new();
        (account_name.len() as u64)
            .le_serialize(&mut serialized_fields)
            .unwrap();
        account_name.le_serialize(&mut serialized_fields).unwrap();
        private_key.io_serialize(&mut serialized_fields).unwrap();
        public_key.io_serialize(&mut serialized_fields).unwrap();
        address.io_serialize(&mut serialized_fields).unwrap();

        let account = Account::new("test", &priv_key_bytes, &pubkey_bytes).unwrap();

        let mut serialized_account: Vec<u8> = Vec::new();
        account.io_serialize(&mut serialized_account).unwrap();

        assert_eq!(serialized_fields, serialized_account);
    }

    #[test]
    fn test_3_correct_account_deserialization() {
        let priv_key_bytes: [u8; 32] = [
            0x0a, 0x52, 0x65, 0x08, 0x2e, 0x24, 0x11, 0x5f, 0x77, 0x54, 0x0a, 0xb3, 0xb8, 0xc2,
            0xb9, 0x20, 0x60, 0xaa, 0x30, 0xd6, 0xd2, 0xb8, 0x1a, 0x08, 0x5d, 0x71, 0xab, 0x37,
            0xed, 0xa7, 0x68, 0x91,
        ];
        let pubkey_bytes: [u8; 33] = [
            0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
            0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
            0x35, 0x14, 0x92, 0x4A, 0x22,
        ];

        let account = Account::new("test", &priv_key_bytes, &pubkey_bytes).unwrap();

        let mut serialized_transaction: Vec<u8> = Vec::new();
        account.io_serialize(&mut serialized_transaction).unwrap();

        let deserialized_account =
            Account::io_deserialize(&mut serialized_transaction.as_slice()).unwrap();

        assert_eq!(account, deserialized_account);
    }

    #[test]
    fn test_04_correct_verify_transaction_ownership() {
        let account_old = Account::new(
            "Old",
            &[
                0x0A, 0x52, 0x65, 0x08, 0x2E, 0x24, 0x11, 0x5F, 0x77, 0x54, 0x0A, 0xB3, 0xB8, 0xC2,
                0xB9, 0x20, 0x60, 0xAA, 0x30, 0xD6, 0xD2, 0xB8, 0x1A, 0x08, 0x5D, 0x71, 0xAB, 0x37,
                0xED, 0xA7, 0x68, 0x91,
            ],
            &[
                0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
                0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
                0x35, 0x14, 0x92, 0x4A, 0x22,
            ],
        )
        .unwrap();

        let transaction_bytes: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, 0x01, 0x20, 0x25, 0xEF, 0x69, 0x2C, 0xA9, 0x87, 0xB3, 0x9A,
            0x81, 0x33, 0x6E, 0xFB, 0x59, 0xB0, 0x56, 0xFB, 0x90, 0xC0, 0x3A, 0x5E, 0xA4, 0xC4,
            0x54, 0x4C, 0xF9, 0x27, 0x57, 0x61, 0x3E, 0x2E, 0xA4, 0x01, 0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0xA0, 0x86, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19,
            0x76, 0xA9, 0x14, 0x7A, 0xA8, 0x18, 0x46, 0x85, 0xCA, 0x1F, 0x06, 0xF5, 0x43, 0xB6,
            0x4A, 0x50, 0x2E, 0xB3, 0xB6, 0x13, 0x5D, 0x67, 0x20, 0x88, 0xAC, 0xD0, 0xE6, 0x43,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xA9, 0x14, 0x4B, 0x88, 0xC1, 0xD3, 0x87,
            0x49, 0x08, 0x36, 0x57, 0x73, 0xA7, 0x65, 0xCD, 0xB0, 0x52, 0xC9, 0xEF, 0x5F, 0x1A,
            0x80, 0x88, 0xAC, 0x98, 0xFB, 0x95, 0x64,
        ];
        let transaction = Transaction::io_deserialize(&mut transaction_bytes.as_slice()).unwrap();

        assert!(account_old.verify_transaction_ownership(&transaction));
    }

    #[test]
    fn test_05_correct_verify_transaction_output_ownership() {
        let account_old = Account::new(
            "Old",
            &[
                0x0A, 0x52, 0x65, 0x08, 0x2E, 0x24, 0x11, 0x5F, 0x77, 0x54, 0x0A, 0xB3, 0xB8, 0xC2,
                0xB9, 0x20, 0x60, 0xAA, 0x30, 0xD6, 0xD2, 0xB8, 0x1A, 0x08, 0x5D, 0x71, 0xAB, 0x37,
                0xED, 0xA7, 0x68, 0x91,
            ],
            &[
                0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
                0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
                0x35, 0x14, 0x92, 0x4A, 0x22,
            ],
        )
        .unwrap();

        let transaction_bytes: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, 0x01, 0x20, 0x25, 0xEF, 0x69, 0x2C, 0xA9, 0x87, 0xB3, 0x9A,
            0x81, 0x33, 0x6E, 0xFB, 0x59, 0xB0, 0x56, 0xFB, 0x90, 0xC0, 0x3A, 0x5E, 0xA4, 0xC4,
            0x54, 0x4C, 0xF9, 0x27, 0x57, 0x61, 0x3E, 0x2E, 0xA4, 0x01, 0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0xA0, 0x86, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19,
            0x76, 0xA9, 0x14, 0x7A, 0xA8, 0x18, 0x46, 0x85, 0xCA, 0x1F, 0x06, 0xF5, 0x43, 0xB6,
            0x4A, 0x50, 0x2E, 0xB3, 0xB6, 0x13, 0x5D, 0x67, 0x20, 0x88, 0xAC, 0xD0, 0xE6, 0x43,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xA9, 0x14, 0x4B, 0x88, 0xC1, 0xD3, 0x87,
            0x49, 0x08, 0x36, 0x57, 0x73, 0xA7, 0x65, 0xCD, 0xB0, 0x52, 0xC9, 0xEF, 0x5F, 0x1A,
            0x80, 0x88, 0xAC, 0x98, 0xFB, 0x95, 0x64,
        ];
        let transaction = Transaction::io_deserialize(&mut transaction_bytes.as_slice()).unwrap();

        let transaction_output = transaction.tx_out[1].clone();

        assert!(account_old.verify_transaction_output_ownership(&transaction_output));
    }
}
