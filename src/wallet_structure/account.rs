use super::{
    error_wallet::ErrorWallet,
    private_key::PrivateKey,
    public_key::PublicKey,
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

use crate::block_structure::transaction_output::TransactionOutput;

pub struct Account {
    pub account_name: String,
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: Address,
}

impl Account {
    pub fn new(name: &str, private_key_bytes: &[u8; 32], public_key_bytes: &[u8; 33], addres: &str) -> Result<Account, ErrorWallet> {
        let account_name = name.to_string();
        let private_key = PrivateKey::new(private_key_bytes)?;
        let public_key = PublicKey::new(public_key_bytes)?;
        let address = Address::new(addres)?;

        Ok(Account {
            account_name,
            private_key,
            public_key,
            address,
        })
    }

    /// Returns true if the account owns the given utxo (works for P2PKH) and false otherwise.
    pub fn verify_transaction_ownership(&self, utxo: TransactionOutput) -> bool {
        let pk_script = utxo.pk_script.clone();
        if pk_script.len() != 25 {
            return false;
        }
        if pk_script[0] != 0x76 || pk_script[1] != 0xa9 || pk_script[2] != 0x14 || pk_script[23] != 0x88 || pk_script[24] != 0xac {
            return false;
        }
        let hashed_pk = &pk_script[3..23];
        hashed_pk == self.address.extract_hashed_pk()
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