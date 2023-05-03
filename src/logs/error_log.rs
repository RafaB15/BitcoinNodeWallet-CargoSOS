
/// Representa los errores posibles para los logs
/// 
/// ### Errores
///  * `ErrorReceiverNotFound`
///  * `ErrorFileNotFound`
///  * `ErrorCouldNotWriteInFile`
#[derive(Debug)]
pub enum ErrorLog {
    ErrorReceiverNotFound,

    ErrorFileNotFound,

    ErrorCouldNotWriteInFile,

}

