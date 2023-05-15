use super::error_initialization::ErrorInitialization;

use cargosos_bitcoin::{
    configurations::error_configuration::ErrorConfiguration,
    connections::error_connection::ErrorConnection, logs::error_log::ErrorLog,
};

use std::fmt::{Debug, Error, Formatter};

use std::convert::From;

pub enum ErrorExecution {
    ErrorInitialization(ErrorInitialization),
    ErrorLog(ErrorLog),
    ErrorConfiguration(ErrorConfiguration),
    ErrorConnection(ErrorConnection),
}

impl Debug for ErrorExecution {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ErrorExecution::ErrorInitialization(error_initialization) => {
                write!(f, "{:?}", error_initialization)
            }
            ErrorExecution::ErrorLog(error_log) => write!(f, "{:?}", error_log),
            ErrorExecution::ErrorConfiguration(error_configuration) => {
                write!(f, "{:?}", error_configuration)
            }
            ErrorExecution::ErrorConnection(error_connection) => {
                write!(f, "{:?}", error_connection)
            }
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
