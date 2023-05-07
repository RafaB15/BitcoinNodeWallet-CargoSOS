
/// Enum to represent the error types we can encounter
/// 
/// ### Errores
///  * `ErrorReceiverNotFound`
///  * `ErrorCouldNotWriteInFile`
#[derive(Debug, PartialEq)]
pub enum ErrorLog {

    ErrorFileNotFound,

    ErrorReceiverNotFound,

    ErrorCouldNotWriteInFile,
}

