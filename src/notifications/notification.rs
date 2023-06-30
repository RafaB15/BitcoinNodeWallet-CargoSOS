use std::net::SocketAddr;

use crate::{
    block_structure::{block::Block, transaction::Transaction},
    messages::command_name::CommandName,
    wallet_structure::account::Account,
};

pub enum Notification {
    AttemptingHandshakeWithPeer(SocketAddr),

    SuccessfulHandshakeWithPeer(SocketAddr),

    FailedHandshakeWithPeer(SocketAddr),

    TransactionOfAccountReceived(Vec<Account>, Transaction),

    TransactionOfAccountInNewBlock(Transaction),

    SuccessfullySentTransaction(Transaction),

    ProgressDownloadingBlocks(u32, u32),

    NewBlockAddedToTheBlockchain(Block),

    UpdatedSelectedAccount(Account),

    RegisterWalletAccount(Account),

    NotifyBlockchainIsReady,

    LoadAvailableBalance(Account, f64, f64),

    AccountNotSelected,

    AccountTransactions(Account, Vec<Transaction>),

    InvalidAddressEnter,

    InvalidPublicKeyEnter,

    InvalidPrivateKeyEnter,

    AccountCreationFail,

    NotEnoughFunds,

    ReceivedMessage(CommandName),

    ClosingPeer,

    ClosingPeers,
}
