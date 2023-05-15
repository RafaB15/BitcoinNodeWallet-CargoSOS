use crate::messages::error_message::ErrorMessage;
use crate::serialization::error_serialization::ErrorSerialization;

#[derive(Debug, PartialEq)]
pub enum ErrorNode {
    WhileSendingMessage(String),
    WhileValidating(String),
    WhileReceivingMessage(String),
    WhileSerializing(String),
    WhileDeserializing(String),
    NodeNotResponding,
}

impl From<ErrorMessage> for ErrorNode {
    fn from(value: ErrorMessage) -> Self {
        match value {
            ErrorMessage::ErrorInMessage => ErrorNode::WhileSendingMessage("Error in message".to_string()),
            ErrorMessage::ErrorInSerialization(error) => ErrorNode::WhileSerializing(error),
            ErrorMessage::ErrorInDeserialization(error) => ErrorNode::WhileDeserializing(error),
            ErrorMessage::ErrorMessageUnknown => ErrorNode::WhileSendingMessage("Error in message".to_string()),
            ErrorMessage::ErrorWhileWriting => ErrorNode::WhileSendingMessage("Error in message".to_string()),
            ErrorMessage::ErrorWhileReading => ErrorNode::WhileReceivingMessage("Error in message".to_string()),
            ErrorMessage::ErrorChecksum => ErrorNode::WhileValidating("Error in message".to_string()),
        }
    }
}

impl From<ErrorSerialization> for ErrorNode {
    fn from(value: ErrorSerialization) -> Self {
        match value {
            ErrorSerialization::ErrorInSerialization(error) => ErrorNode::WhileSerializing(error),
            ErrorSerialization::ErrorInDeserialization(error) => ErrorNode::WhileDeserializing(error),
            ErrorSerialization::ErrorWhileWriting => ErrorNode::WhileSendingMessage("Error in message".to_string()),
            ErrorSerialization::ErrorWhileReading => ErrorNode::WhileReceivingMessage("Error in message".to_string()),
        }
    }
}