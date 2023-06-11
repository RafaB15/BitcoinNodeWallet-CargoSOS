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
    block_chain::BlockChain,
    transaction_output::TransactionOutput,
    utxo_set::UTXOSet,
};

#[derive(Debug)]
pub struct Account {
    pub account_name: String,
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: Address,
    pub utxo_set: UTXOSet,
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
        let public_key = PublicKey::new(public_key_bytes)?;
        let address = Address::new(addres)?;
        let utxo_set = UTXOSet::new(Some(address.clone()));

        Ok(Account {
            account_name,
            private_key,
            public_key,
            address,
            utxo_set,
        })
    }

    /// Returns true if the account owns the given utxo (works for P2PKH) and false otherwise.
    pub fn verify_transaction_ownership(&self, utxo: &TransactionOutput) -> bool {
        self.address.verify_transaction_ownership(utxo)
    }

    /// Returns the balance of the account
    pub fn get_balance(&self) -> i64 {
        self.utxo_set.get_balance()
    }

    /// Initializes the utxo set of the account from the blockchain
    pub fn initialize_utxo_form_blockchain_utxo(&mut self, utxo_set: &UTXOSet) -> Result<(), ErrorWallet> {
        self.utxo_set = UTXOSet::from_utxo_set(utxo_set, &self.address);
        Ok(())
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

        let account_name = String::deserialize_fix_size(stream, account_name_len)?;
        let private_key = PrivateKey::io_deserialize(stream)?;
        let public_key = PublicKey::io_deserialize(stream)?;
        let address = Address::io_deserialize(stream)?;
        let utxo_set = UTXOSet::new(Some(address.clone()));

        Ok(Account{
            account_name,
            private_key,
            public_key,
            address,
            utxo_set,
        })
    }
}