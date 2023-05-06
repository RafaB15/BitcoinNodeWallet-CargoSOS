
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionError {
    ErrorInvalidInputParse,
    ErrorInvalidIPOrPortNumber,
    ErrorCannotConnectToAddress,
    ErrorCannotObtainOwnAddress,
    ErrorCannotSendMessage,
}