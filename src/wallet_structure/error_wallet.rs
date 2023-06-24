/// It represents all the possible error that can appear interacting with the wallet
#[derive(Debug, std::cmp::PartialEq)]
pub enum ErrorWallet {
    /// It will appear when private key for an account cannot be generated
    CannotGeneratePrivateKey(String),

    /// It will appear when public key for an account cannot be generated
    CannotGeneratePublicKey(String),

    /// It will appear when address for an account cannot be generated
    CannotDecodeAddress(String),

    /// It will appear when a transaction cannot be created
    CannotCreateNewTransaction(String),

    /// It will appear when a transaction cannot be signed
    CannotSignMessage(String),

    /// It will appear when an account does not have enough funds to create a transaction for the amount requested
    NotEnoughFunds(String),

    /// It will appear when a problem appears when trying to create an address from a public key
    CannotCreateAddress(String),
}
