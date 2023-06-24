/// It represents all posible errors that can occur in the connection to a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorConnection {
    /// It will appear if the IP or the port number its not valid
    ErrorInvalidIPOrPortNumber,

    /// It will appear when the connection is not established with a peer
    ErrorCannotConnectToAddress,

    /// It will appear when a message to a peer cannot be sent
    ErrorCannotSendMessage,

    /// It will appear when a message from a peer cannot be received
    ErrorCannotReceiveMessage,
}
