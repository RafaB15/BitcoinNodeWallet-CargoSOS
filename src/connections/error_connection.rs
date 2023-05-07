
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorConnection {
    ErrorInvalidInputParse,
    ErrorInvalidIPOrPortNumber,
    ErrorCannotConnectToAddress,
    ErrorCannotObtainOwnAddress,
    ErrorCannotSendMessage,
    ErrorCannotReceiveMessage
}