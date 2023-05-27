#[derive(Debug, std::cmp::PartialEq)]
pub enum ErrorWallet {
    CannotGeneratePrivateKey(String),
    CannotGeneratePublicKey(String),
}