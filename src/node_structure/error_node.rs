use crate::messages::error_message::ErrorMessage;

#[derive(Debug, PartialEq)]
pub enum ErrorNode {
    ErrorWhileSendingMessage(String),
    ErrorWhileValidating(String),
}

impl From<ErrorMessage> for ErrorNode {
    fn from(value: ErrorMessage) -> Self {
        match value {
            ErrorMessage::ErrorInMessage => ErrorNode::ErrorWhileSendingMessage("Error in message".to_string()),
            ErrorMessage::ErrorInSerialization(error) => ErrorNode::ErrorWhileSendingMessage(format!("Error in serialization: {}", error)),
            ErrorMessage::ErrorInDeserialization(error) => ErrorNode::ErrorWhileSendingMessage(format!("Error in deserialization: {}", error)),
            ErrorMessage::ErrorMessageUnknown => ErrorNode::ErrorWhileSendingMessage("Error message unknown".to_string()),
            ErrorMessage::ErrorWhileWriting => ErrorNode::ErrorWhileSendingMessage("Error while writing".to_string()),
            ErrorMessage::ErrorWhileReading => ErrorNode::ErrorWhileSendingMessage("Error while reading".to_string()),
            ErrorMessage::ErrorChecksum => ErrorNode::ErrorWhileSendingMessage("Error checksum".to_string()),
        }
    }
}