use super::{
    error_wallet::ErrorWallet,
    private_key::{PrivateKey, PrivateKeyType},
    public_key::{PublicKey, PublicKeyType},
    address::Address,
};

use crate::serialization::{
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_fix_size::DeserializableFixSize,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
};

use std::io::{Read, Write};

use crate::block_structure::{
    transaction::Transaction,
    transaction_output::TransactionOutput,
    outpoint::Outpoint,
    utxo_set::UTXOSet,
};

#[derive(Debug)]
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
        addres: &str
    ) -> Result<Account, ErrorWallet> {
        let account_name = name.to_string();
        let private_key = PrivateKey::new(private_key_bytes)?;
        let public_key = PublicKey::new(public_key_bytes);
        let address = Address::new(addres)?;

        Ok(Account {
            account_name,
            private_key,
            public_key,
            address,
        })
    }

    /// Returns true if the account owns the given utxo (works for P2PKH) and false otherwise.
    pub fn verify_transaction_ownership(&self, utxo: &TransactionOutput) -> bool {
        self.address.verify_transaction_ownership(utxo)
    }

    /// Returns the balance of the account in satoshis
    pub fn get_balance_in_satoshis(&self, utxo_set: UTXOSet) -> i64 {
        utxo_set.get_balance_in_satoshis(&self.address)
    }

    /// Returns the balance of the account in tbtc
    pub fn get_balance_in_tbtc(&self, utxo_set: UTXOSet) -> f64 {
        utxo_set.get_balance_in_tbtc(&self.address)
    }
    
    pub fn create_transaction(&self, to: Address, amount: i64, fee: i64, utxo_set: UTXOSet) -> Result<Transaction, ErrorWallet> {
        let mut available_outputs = utxo_set.get_utxo_list_with_outpoints(Some(&self.address));
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
        
        match Transaction::from_account_to_address(
            &self,
            &outputs_to_spend,
            &to,
            amount,
            fee,
        ) {
            Ok(transaction) => {
                Ok(transaction)
            },
            Err(error) => {
                Err(ErrorWallet::CannotCreateNewTransaction(format!("Error while trying to create a new transaction. Error: {:?}", error)))
            }
        }

    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ErrorWallet> {
        self.private_key.sign(message)
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

        Ok(Account{
            account_name: String::deserialize_fix_size(stream, account_name_len)?,
            private_key: PrivateKey::io_deserialize(stream)?,
            public_key: PublicKey::io_deserialize(stream)?,
            address: Address::io_deserialize(stream)?,
        })
    }
}
/* 
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_1_we_can_create_a_transaction() {
        let account_1 = Account::new(
            "Account 1",
            [0x0B, 0x1F, 0xDE, 0x5B, 0xA8, 0xAA, 0xD5, 0xB4, 0x4E, 0x66, 0x0B, 0xD2, 0xB3, 0x29, 0x3D, 0xFC, 0x8C, 0x76, 0xC4, 0xB1, 0xCE, 0xCE, 0x89, 0x95, 0x5A, 0xB0, 0x7E, 0xFE, 0xEF, 0x39, 0xB4, 0x5B],
            [0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68, 0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2, 0x35, 0x14, 0x92, 0x4A, 0x22],
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        ).unwrap();

        
    }
}
*/