use super::{
    error_wallet::ErrorWallet,
    private_key::PrivateKey,
    public_key::PublicKey,
    address::Address,
};

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
        let hashed_pk = &pk_script[3..23];
        hashed_pk == self.address.extract_hashed_pk()
    }
}