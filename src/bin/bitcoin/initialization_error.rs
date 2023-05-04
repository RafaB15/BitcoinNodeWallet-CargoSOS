use cargosos_bitcoin::configurations::parse_error::ParseError;
use std::fmt::Display;
use std::convert::From;

pub enum InitializationError {
    /// It will appear when there is not `-c`, `--config` or `--configuration` in the arguments or there is not argument pass that configuration declaration
    ErrorNoGivenFile,
    
    /// It will appear when the file does not exist
    ErrorFileNotExist,

    /// It will appear when there given readable gives an error when read
    ErrorFileError,

    /// It will appear when there isn't a configuration at all
    ErrorIncompleteConfiguration,

    /// It will appear when there isn't a structure with a given property name
    ErrorConfigurationNotFound
}

impl Display for InitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            InitializationError::ErrorFileNotExist => "Configuration file doesn't exist",
            InitializationError::ErrorNoGivenFile => "Not given a configuration file",
            InitializationError::ErrorFileError => "Configuration file gives an error",
            InitializationError::ErrorIncompleteConfiguration => "Configuration not complete",
            InitializationError::ErrorConfigurationNotFound => "There is not a configuration in the configuration file",
        };

        let message = format!("An error ocurre: {}", message);
        write!(f, "{message}")
    }
}

impl From<ParseError> for InitializationError {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::ErrorIncompleteConfiguration => InitializationError::ErrorIncompleteConfiguration,
            ParseError::ErrorConfigurationNoFount => InitializationError::ErrorConfigurationNotFound,
            _ => InitializationError::ErrorFileError,
        }
    }
}