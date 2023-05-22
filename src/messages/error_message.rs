use std::convert::From;

use crate::serialization::error_serialization::ErrorSerialization;

/// Enum to represent the error types we can encounter in messages
///
/// ### Errores
///  * 'ErrorInMessage'
///  * 'ErrorInSerialization'
#[derive(Debug, PartialEq)]
pub enum ErrorMessage {
    InMessage,

    InSerialization(String),

    InDeserialization(String),

    MessageUnknown,

    WhileWriting,

    WhileReading,

    Checksum,

    RequestedDataTooBig,
}

impl From<ErrorSerialization> for ErrorMessage {
    fn from(value: ErrorSerialization) -> Self {
        match value {
            ErrorSerialization::ErrorInSerialization(error) => ErrorMessage::InSerialization(error),
            ErrorSerialization::ErrorInDeserialization(error) => {
                ErrorMessage::InDeserialization(error)
            }
            ErrorSerialization::ErrorWhileWriting => ErrorMessage::WhileWriting,
            ErrorSerialization::ErrorWhileReading => ErrorMessage::WhileReading,
        }
    }
}
