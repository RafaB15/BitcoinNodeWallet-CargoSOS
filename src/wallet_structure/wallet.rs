use super::{
    account::Account,
    error_wallet::ErrorWallet,
};

use crate::{
    configurations::try_default::TryDefault,
    serialization::{
        serializable_internal_order::SerializableInternalOrder,
        serializable_little_endian::SerializableLittleEndian,
        deserializable_internal_order::DeserializableInternalOrder,
        deserializable_little_endian::DeserializableLittleEndian,
        error_serialization::ErrorSerialization,
    }
};

use std::io::{Read, Write};

#[derive(Debug)]
pub struct Wallet {
    pub accounts: Vec<Account>,
}

impl Wallet {
    pub fn new(accounts: Vec<Account>) -> Wallet {
        Wallet {
            accounts,
        }
    }

    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
    }

    pub fn get_accounts(&self) -> &Vec<Account> {
        &self.accounts
    }
}

impl TryDefault for Wallet {
    type Error = ErrorWallet;

    fn try_default() -> Result<Self, Self::Error> {
        Ok(Wallet::new(Vec::new()))
    }
}

impl SerializableInternalOrder for Wallet {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        (self.accounts.len() as u64).le_serialize(stream)?;

        for account in &self.accounts {
            account.io_serialize(stream)?;
        }

        Ok(())
    }
}

impl DeserializableInternalOrder for Wallet {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let accounts_len = u64::le_deserialize(stream)?;

        let mut accounts: Vec<Account> = Vec::new();
        for _ in 0..accounts_len {
            accounts.push(Account::io_deserialize(stream)?);
        }

        Ok(Wallet::new(accounts))
    }
}