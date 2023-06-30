/// It represents all posible errors that can occur in the block chain, and related structures
#[derive(Debug)]
pub enum ErrorBlock {
    /// It will appear when the transaction id could not be created
    CouldNotGetTxId,

    /// It will appear when the transaction id could not be written
    CouldNotWriteTxId(String),

    /// It will appear when the Transaction is already in the block
    TransactionAlreadyInBlock,

    /// It will appear when the proof of work of a header is not valid
    ErrorWithProofOfWork,

    /// It will appear when a header could not be hash correctly
    CouldNotHash,

    /// It will appear when the block could not be appended to the block chain
    CouldNotAppendBlock,

    /// It will appear when the block chain could not be updated with a block or header
    CouldNotUpdate,

    /// It will appear when a node position it's not found in the block chain
    NodeChainReferenceNotFound,

    /// It will appear when the merkle path could not be calculated
    CouldNotCalculateMerklePath,

    /// It will appear when the transaction could not be found
    TransactionNotFound,

    /// It will appear when the root hash in merkle tree could not be found
    RootHashNotFound,

    /// It will appear when the hash in merkle tree could not be found at given index
    NoHashFound,
}
