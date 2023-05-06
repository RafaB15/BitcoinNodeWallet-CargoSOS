#[derive(Debug)]
pub enum ErrorInitialization {
    /// It will appear when there is not `-c`, `--config` or `--configuration` in the arguments or there is not argument pass that configuration declaration
    ErrorNoGivenConfigurationFile,
    
    /// It will appear when the file does not exist
    ErrorConfigurationFileDoesntExist
}