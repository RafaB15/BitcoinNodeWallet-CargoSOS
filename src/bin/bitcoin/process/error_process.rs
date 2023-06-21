use cargosos_bitcoin::serialization::error_serialization::ErrorSerialization;

use std::convert::From;

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
