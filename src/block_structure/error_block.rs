#[derive(Debug)]
pub enum ErrorBlock {
    /// This will appear when the TxId could not be created
    CouldNotGetTxId,

    /// This will appear when the TxId could not be written
    CouldNotWriteTxId,

    ///This will appear when the Transaction is already in the block
    TransactionAlreadyInBlock,

    ErrorWithProofOfWork,

    CouldNotSerialize,

    CouldNotHash,

    CouldNotAppendBlock,

    CouldNotUpdate,

    CouldNotFindBlockFarEnough,

    NodeChainReferenceNotFound,

    ///This will appear when the merkle path could not be calculated
    CouldNotCalculateMerklePath,

    ///This will appear when the transaction could not be found
    NoTransactions,

    ///This will appear when the transaction could not be found
    TransactionNotFound,

    ///This will appear when the root hash in merkle tree could not be found
    RootHashNotFound,

    ///This will appear when the hash in merkle tree could not be found at given index
    NoHashFound,
}
