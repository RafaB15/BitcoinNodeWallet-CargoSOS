
#[derive(Debug)]
pub enum ErrorInitialization {
    /// It will appear when there is not argument pass that configuration declaration
    NoGivenConfigurationFile,
    
    /// It will appear when the file does not exist
    ConfigurationFileDoesntExist,

    /// It will appear when the file does not exist
    LogFileDoesntExist,

    ///It will appear when the file could not be truncated
    ErrorCouldNotTruncateFile
}