use std::net::SocketAddr;

use crate::{
    block_structure::{block::Block, hash::HashType, transaction::Transaction},
    messages::command_name::CommandName,
    node_structure::connection_id::ConnectionId,
    wallet_structure::account::Account,
};

/// The different types of notifications that the notifier can send.
pub enum Notification {
    /// Notifies that we are trying to establish a connection with a peer.
    AttemptingHandshakeWithPeer(SocketAddr),

    /// Notifies that we have successfully established a connection with a peer.
    SuccessfulHandshakeWithPeer(SocketAddr),

    /// Notifies that we have failed to establish a connection with a peer.
    FailedHandshakeWithPeer(SocketAddr),

    /// Notifies that we have to update a connection
    ConnectionUpdated(ConnectionId),

    /// Notifies that we have received a transaction for an account in the wallet.
    TransactionOfAccountReceived(Vec<Account>, Transaction),

    /// Notifies that there was a problem while trying to obtain the merkle proof of inclusion.
    ProblemVerifyingTransactionMerkleProofOfInclusion(String),

    /// Notifies that we have received a transaction for an account in the wallet in a block.
    TransactionOfAccountInNewBlock(Transaction),

    /// Notifies that we have successfully sent a transaction.
    SuccessfullySentTransaction(Transaction),

    /// Notifies that we successfully obtained the merkle poof of inclusion.
    SuccessfulMerkleProof(Vec<HashType>, HashType),

    /// Notifies that we have received an amount of headers.
    HeadersReceived(u32),

    /// Notifies the amount of blocks that we have downloaded.
    ProgressDownloadingBlocks(u32, u32),

    /// Notifies the amount of blocks added to the blockchain.
    ProgressUpdatingBlockchain(u32, u32),

    /// Notifies that we have received a block.
    NewBlockAddedToTheBlockchain(Block),

    /// Notifies that we have updated the selected account.
    UpdatedSelectedAccount(Account),

    /// Notifies that we have registered a new account.
    RegisterWalletAccount(Account),

    /// Notifies that the blockchain is ready.
    NotifyBlockchainIsReady,

    /// Notifies the balance of an account
    LoadAvailableBalance(Account, f64, f64),

    /// Notifies that there is no account currently selected.
    AccountNotSelected,

    /// Notifies the information of the transactions of an account.
    AccountTransactions(Account, Vec<Transaction>),

    /// Notifies that we have entered an invalid address.
    InvalidAddressEnter,

    /// Notifies that we have entered an invalid public key.
    InvalidPublicKeyEnter,

    /// Notifies that we have entered an invalid private key.
    InvalidPrivateKeyEnter,

    /// Notifies that we have failed to create an account.
    AccountCreationFail,

    /// Notifies that we do not have enough funds to create a transaction.
    NotEnoughFunds,

    /// Notifies that we have received a message.
    ReceivedMessage(CommandName),

    /// Notifies that we are closing a peer.
    ClosingPeer,

    /// Notifies that we are closing all peers.
    ClosingPeers,
}
