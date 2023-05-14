use bitcoin_hashes::hex::Error;

#[derive(Debug)]
pub enum ErrorInitialization {
    /// It will appear when there is not `-c`, `--config` or `--configuration` in the arguments or there is not argument pass that configuration declaration
    ErrorNoGivenConfigurationFile,
    
    /// It will appear when the file does not exist
    ErrorConfigurationFileDoesntExist,

    /// It will appear when the file does not exist
    ErrorLogFileDoesntExist,

    ///It will appear when the file could not be truncated
    ErrorCouldNotTruncateFile
}












/*impl From<ErrorSerialization> for ErrorInitialization {
    fn from(value: ErrorSerialization) -> Self {
        match value {

            ErrorSerialization::ErrorInSerialization(error) => ErrorMessage::ErrorInSerialization(error),
            ErrorSerialization::ErrorInDeserialization(error) => ErrorMessage::ErrorInDeserialization(error),
            ErrorSerialization::ErrorWhileWriting => ErrorMessage::ErrorWhileWriting,
            ErrorSerialization::ErrorWhileReading => ErrorMessage::ErrorWhileReading,
        }
    }
}*/