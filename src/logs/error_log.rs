/// It represents all posible errors that can occur in the logs
#[derive(Debug, PartialEq)]
pub enum ErrorLog {
    /// It will appear when the file does not exist
    FileNotFound,

    /// It will appear when the receiver it's drop and can't send the message
    ReceiverNotFound,

    /// It will appear when no more lines can be added to the given file
    CouldNotWriteInFile,
}
