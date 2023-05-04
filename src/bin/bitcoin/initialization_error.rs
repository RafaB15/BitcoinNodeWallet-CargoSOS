
pub enum InitializationError {
    /// It will appear when there is not `-c`, `--config` or `--configuration` in the arguments or there is not argument pass that configuration declaration
    ErrorNoGivenFile,
    
    /// It will appear when the file does not exist
    ErrorFileNotExist,
}

impl std::fmt::Display for InitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            InitializationError::ErrorFileNotExist => "Configuration file doesn't exist",
            InitializationError::ErrorNoGivenFile => "Not given a configuration file",
        };

        let message = format!("An error ocurre: {}", message);
        write!(f, "{message}")
    }
}