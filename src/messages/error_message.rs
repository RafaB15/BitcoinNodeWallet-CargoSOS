use std::convert::From;

use crate::serialization::error_serialization::ErrorSerialization;

/// Enum to represent the error types we can encounter in messages
///
/// ### Errores
///  * 'ErrorInMessage'
///  * 'ErrorInSerialization'
#[derive(Debug, PartialEq)]
pub enum ErrorMessage {
    ErrorInMessage,

    ErrorInSerialization(String),

    ErrorInDeserialization(String),

    ErrorMessageUnknown,

    ErrorWhileWriting,

    ErrorWhileReading,

    ErrorChecksum,
}

impl From<ErrorSerialization> for ErrorMessage {
    fn from(value: ErrorSerialization) -> Self {
        match value {
            ErrorSerialization::ErrorInSerialization(error) => {
                ErrorMessage::ErrorInSerialization(error)
            }
            ErrorSerialization::ErrorInDeserialization(error) => {
                ErrorMessage::ErrorInDeserialization(error)
            }
            ErrorSerialization::ErrorWhileWriting => ErrorMessage::ErrorWhileWriting,
            ErrorSerialization::ErrorWhileReading => ErrorMessage::ErrorWhileReading,
        }
    }
}
