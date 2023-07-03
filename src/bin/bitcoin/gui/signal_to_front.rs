use cargosos_bitcoin::{
    node_structure::connection_id::ConnectionId,
    block_structure::hash::HashType,
};

/// This enum represents the signal that the back sends to the front.
pub enum SignalToFront {
    /// Signal to add an account to the list of accounts.
    RegisterAccount(String),

    /// Signal to update the available balance.
    LoadAvailableBalance((f64, f64)),

    /// Signal to notify that the blockchain is ready.
    NotifyBlockchainIsReady,

    /// Signal to notify that an error involving an account occurred.
    ErrorInTransaction(String),

    /// Signal to notify that an error ocurred while creating an account.
    ErrorInAccountCreation(String),

    /// Signal to notify that we received a transaction from one of our accounts.
    TransactionOfAccountReceived(String),

    /// Signal to notify that we received a transaction from one of our accounts in a block.
    BlockWithUnconfirmedTransactionReceived,

    /// Signal to transmit the information of the transactions of an account.
    AccountTransactions(Vec<(u32, HashType, i64)>),

    /// Signal to indicate that an error occurred while trying to get the merkle proof of inclusion
    ErrorInMerkleProof(String),

    /// SIgnal to transmit information about the merkle path of a transaction in a block.
    DisplayMerklePath(Vec<HashType>, HashType),

    /// Signal to notify that we have to update the progress bar download of blocks.
    UpdateBlockProgressBar(u32, u32),

    /// Signal to notify that we have to update the progress bar update of the blockchain.
    UpdateBlockchainProgressBar(u32, u32),

    /// Signal to notify to that we have to update the current connections
    UpdateConnection(ConnectionId),

    /// Signal to notify that we have to update the front.
    Update,
}
