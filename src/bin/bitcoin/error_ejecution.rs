use super::error_initialization::ErrorInitialization;

use cargosos_bitcoin::{
    logs::error_log::ErrorLog,
    configurations::parse_error::ParseError,
};

use std::fmt::{
    Debug,
    Formatter,
    Error,
};

use std::convert::From;

pub enum ErrorEjecution {
    ErrorInitialization(ErrorInitialization),
    ErrorLog(ErrorLog),
    ParseError(ParseError),
}

impl Debug for ErrorEjecution {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ErrorEjecution::ErrorInitialization(error_initialization) => write!(f, "{:?}", error_initialization),
            ErrorEjecution::ErrorLog(error_log) => write!(f, "{:?}", error_log),
            ErrorEjecution::ParseError(parse_error) => write!(f, "{:?}", parse_error),
        }
    }
}

impl From<ErrorInitialization> for ErrorEjecution {
    fn from(value: ErrorInitialization) -> Self {
        ErrorEjecution::ErrorInitialization(value)
    }
}

impl From<ErrorLog> for ErrorEjecution {
    fn from(value: ErrorLog) -> Self {
        ErrorEjecution::ErrorLog(value)
    }
}

impl From<ParseError> for ErrorEjecution {
    fn from(value: ParseError) -> Self {
        ErrorEjecution::ParseError(value)
    }
}