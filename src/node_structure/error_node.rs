use crate::messages::error_message::ErrorMessage;
use crate::serialization::error_serialization::ErrorSerialization;

#[derive(Debug, PartialEq)]
pub enum ErrorNode {
    WhileSendingMessage(String),
    WhileValidating(String),
    WhileReceivingMessage(String),
    WhileSerializing(String),
    WhileDeserializing(String),
    NodeNotResponding(String),
    RequestedDataTooBig,
}

impl From<ErrorMessage> for ErrorNode {
    fn from(value: ErrorMessage) -> Self {
        match value {
            ErrorMessage::InMessage => ErrorNode::WhileSendingMessage("Error in message".to_string()),
            ErrorMessage::InSerialization(error) => ErrorNode::WhileSerializing(error),
            ErrorMessage::InDeserialization(error) => ErrorNode::WhileDeserializing(error),
            ErrorMessage::MessageUnknown => ErrorNode::WhileSendingMessage("Error in message".to_string()),
            ErrorMessage::WhileWriting => ErrorNode::WhileSendingMessage("Error in message".to_string()),
            ErrorMessage::WhileReading => ErrorNode::WhileReceivingMessage("Error in message".to_string()),
            ErrorMessage::Checksum => ErrorNode::WhileValidating("Error in message".to_string()),
            ErrorMessage::RequestedDataTooBig => ErrorNode::RequestedDataTooBig,
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