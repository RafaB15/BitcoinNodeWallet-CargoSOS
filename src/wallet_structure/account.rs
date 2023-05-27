use super::{
    error_wallet::ErrorWallet,
    private_key::PrivateKey,
    public_key::PublicKey,
    address::Address,
};

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
}