use super::error_initialization::ErrorInitialization;

use cargosos_bitcoin::{
    logs::error_log::ErrorLog,
    configurations::error_configuration::ErrorConfiguration,
    connections::error_connection::ErrorConnection,
    messages::error_message::ErrorMessage,
    block_structure::error_block::ErrorBlock,
};

use std::fmt::{
    Debug,
    Formatter,
    Error,
};

use std::convert::From;

pub enum ErrorExecution {
    ErrorInitialization(ErrorInitialization),
    ErrorLog(ErrorLog),
    ErrorConfiguration(ErrorConfiguration),
    ErrorConnection(ErrorConnection),
    ErrorMessage(ErrorMessage),
    ErrorBlock(ErrorBlock),

    ErrorFailThread,
}

impl Debug for ErrorExecution {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ErrorExecution::ErrorInitialization(error_initialization) => write!(f, "{:?}", error_initialization),
            ErrorExecution::ErrorLog(error_log) => write!(f, "{:?}", error_log),
            ErrorExecution::ErrorConfiguration(error_configuration) => write!(f, "{:?}", error_configuration),
            ErrorExecution::ErrorConnection(error_connection) => write!(f, "{:?}", error_connection),
            ErrorExecution::ErrorMessage(error_message) => write!(f, "{:?}", error_message),
            ErrorExecution::ErrorBlock(error_block) => write!(f, "{:?}", error_block),
            ErrorExecution::ErrorFailThread => write!(f, "ErrorFailThread"),
        }
    }
}

impl From<ErrorInitialization> for ErrorExecution {
    fn from(value: ErrorInitialization) -> Self {
        ErrorExecution::ErrorInitialization(value)
    }
}

impl From<ErrorLog> for ErrorExecution {
    fn from(value: ErrorLog) -> Self {
        ErrorExecution::ErrorLog(value)
    }
}

impl From<ErrorConfiguration> for ErrorExecution {
    fn from(value: ErrorConfiguration) -> Self {
        ErrorExecution::ErrorConfiguration(value)
    }
}

impl From<ErrorConnection> for ErrorExecution {
    fn from(value: ErrorConnection) -> Self {
        ErrorExecution::ErrorConnection(value)
    }
}

impl From<ErrorMessage> for ErrorExecution {
    fn from(value: ErrorMessage) -> Self {
        ErrorExecution::ErrorMessage(value)
    }
}

impl From<ErrorBlock> for ErrorExecution {
    fn from(value: ErrorBlock) -> Self {
        ErrorExecution::ErrorBlock(value)
    }
}