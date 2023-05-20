#[derive(Debug)]
pub enum ErrorBlock {
    /// This will appear when the TxId could not be created
    CouldNotGetTxId,

    /// This will appear when the TxId could not be written
    CouldNotWriteTxId,

    ///This will appear when the Transaction is already in the block
    TransactionAlreadyInBlock,

    CouldNotSerialize,

    CouldNotHash,

    CouldNotAppendBlock,

    CouldNotUpdate,

    CouldNotFindBlockFarEnough,

    NodeChainReferenceNotFound,

    CouldNotCalculateMerklePath,

    NoTransactions,

    TransactionNotFound,
}

