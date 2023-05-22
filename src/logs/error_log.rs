/// Enum to represent the error types we can encounter
///
/// ### Errores
///  * `ReceiverNotFound`
///  * `CouldNotWriteInFile`
///  * `FileNotFound`
#[derive(Debug, PartialEq)]
pub enum ErrorLog {
    FileNotFound,

    ReceiverNotFound,

    CouldNotWriteInFile,
}
