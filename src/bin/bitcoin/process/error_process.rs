use cargosos_bitcoin::serialization::error_serialization::ErrorSerialization;

use std::convert::From;

#[derive(Debug, Clone)]
pub enum ErrorProcess {
    ErrorReading,
    ErrorWriting,
    FailThread,
    ErrorFromPeer(String),
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
