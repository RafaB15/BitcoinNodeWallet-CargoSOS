use cargosos_bitcoin::serialization::error_serialization::ErrorSerialization;

use std::convert::From;

/// It represents all posible errors that can occur in the process of connecting with a peer
#[derive(Debug, Clone)]
pub enum ErrorProcess {
    /// It will appear while reading from the stream
    ErrorReading,

    /// It will appear while writing to the stream
    ErrorWriting,

    /// It will appear when a thread panics and fails
    FailThread,

    /// It will appear when a conextion with a peer fails
    ErrorFromPeer(String),

    /// It will appear when can't create the default value
    CannotCreateDefault,

    /// It will appear when try to get a value that is already loaded
    AlreadyLoaded,

    /// It will appear when we try to unwrap an Arc
    CannotUnwrapArc,

    /// It will appear when we try to get the inner value of a mutex
    CannotGetInner,

    /// It will appear when trying to create a transaction of an amount and fee greater than the balance
    TransactionWithoutSufficientFunds,

    /// It will appear when trying to create a transaction and fails to create the signature script for it
    TransactionCreationFail,
}

impl From<ErrorSerialization> for ErrorProcess {
    fn from(error: ErrorSerialization) -> Self {
        match error {
            ErrorSerialization::ErrorInSerialization(_) => ErrorProcess::ErrorWriting,
            ErrorSerialization::ErrorInDeserialization(_) => ErrorProcess::ErrorReading,
            ErrorSerialization::ErrorWhileWriting => ErrorProcess::ErrorWriting,
            ErrorSerialization::ErrorWhileReading => ErrorProcess::ErrorReading,
        }
    }
}
