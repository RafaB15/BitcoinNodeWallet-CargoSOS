use crate::messages::error_message::ErrorMessage;
use crate::serialization::error_serialization::ErrorSerialization;

#[derive(Debug, PartialEq)]
pub enum ErrorNode {
    ErrorWhileSendingMessage(String),
    ErrorWhileValidating(String),
    ErrorWhileReceivingMessage(String),
    ErrorWhileSerializing(String),
    ErrorWhileDeserializing(String),
}

impl From<ErrorMessage> for ErrorNode {
    fn from(value: ErrorMessage) -> Self {
        match value {
            ErrorMessage::ErrorInMessage => ErrorNode::ErrorWhileSendingMessage("Error in message".to_string()),
            ErrorMessage::ErrorInSerialization(error) => ErrorNode::ErrorWhileSerializing(error),
            ErrorMessage::ErrorInDeserialization(error) => ErrorNode::ErrorWhileDeserializing(error),
            ErrorMessage::ErrorMessageUnknown => ErrorNode::ErrorWhileSendingMessage("Error in message".to_string()),
            ErrorMessage::ErrorWhileWriting => ErrorNode::ErrorWhileSendingMessage("Error in message".to_string()),
            ErrorMessage::ErrorWhileReading => ErrorNode::ErrorWhileReceivingMessage("Error in message".to_string()),
            ErrorMessage::ErrorChecksum => ErrorNode::ErrorWhileValidating("Error in message".to_string()),
        }
    }
}

impl From<ErrorSerialization> for ErrorNode {
    fn from(value: ErrorSerialization) -> Self {
        match value {
            ErrorSerialization::ErrorInSerialization(error) => ErrorNode::ErrorWhileSerializing(error),
            ErrorSerialization::ErrorInDeserialization(error) => ErrorNode::ErrorWhileDeserializing(error),
            ErrorSerialization::ErrorWhileWriting => ErrorNode::ErrorWhileSendingMessage("Error in message".to_string()),
            ErrorSerialization::ErrorWhileReading => ErrorNode::ErrorWhileReceivingMessage("Error in message".to_string()),
        }
    }
}