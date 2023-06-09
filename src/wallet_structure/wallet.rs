use crate::configurations::try_default::TryDefault;
use super::{
    account::Account,
    error_wallet::ErrorWallet,
};

pub struct Wallet {
    accounts: Vec<Account>,
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