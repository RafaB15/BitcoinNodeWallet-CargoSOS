/// Enum to represent the error types we can encounter in messages
/// 
/// ### Errores
///  * 'ErrorInMessage'
///  * 'ErrorInSerialization'
#[derive(Debug, PartialEq)]
pub enum ErrorMessage {

    ErrorInMessage,

    ErrorInSerialization
}

