use crate::serialization::error_serialization::ErrorSerialization;

/// It represents all posible errors that can occur while making the protocols of a node
#[derive(Debug, PartialEq)]
pub enum ErrorNode {
    /// It will appear when there is an error while sending a message to a peer or others threads
    WhileSendingMessage(String),

    /// It will appear when a given header does not pass the proof of work to be added to the blockchain
    WhileValidating(String),

    /// It will appear when there is an error in the reading from a stream
    WhileReceivingMessage(String),

    /// It will appear when there is an error in the serialization
    WhileSerializing(String),

    /// It will appear when there is an error in the deserialization
    WhileDeserializing(String),

    /// It will appear when the node is not responding to the messages
    NodeNotResponding(String),

    /// It will appear when the headers count is bigger than the maximum headers count
    RequestedDataTooBig,
}

impl From<ErrorSerialization> for ErrorNode {
    fn from(value: ErrorSerialization) -> Self {
        match value {
            ErrorSerialization::ErrorInSerialization(error) => ErrorNode::WhileSerializing(error),
            ErrorSerialization::ErrorInDeserialization(error) => {
                ErrorNode::WhileDeserializing(error)
            }
            ErrorSerialization::ErrorWhileWriting => {
                ErrorNode::WhileSendingMessage("Error in message".to_string())
            }
            ErrorSerialization::ErrorWhileReading => {
                ErrorNode::WhileReceivingMessage("Error in message".to_string())
            }
        }
    }
}
