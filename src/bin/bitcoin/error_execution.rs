use super::error_initialization::ErrorInitialization;

use cargosos_bitcoin::{
    block_structure::error_block::ErrorBlock,
    configurations::error_configuration::ErrorConfiguration,
    connections::error_connection::ErrorConnection, logs::error_log::ErrorLog,
    messages::error_message::ErrorMessage, node_structure::error_node::ErrorNode,
    serialization::error_serialization::ErrorSerialization,
    wallet_structure::error_wallet::ErrorWallet,
};

use std::fmt::{Debug, Error, Formatter};

use std::convert::From;

pub enum ErrorExecution {
    Initialization(ErrorInitialization),
    Log(ErrorLog),
    Configuration(ErrorConfiguration),
    Connection(ErrorConnection),
    Serialization(ErrorSerialization),
    Message(ErrorMessage),
    Block(ErrorBlock),
    Node(ErrorNode),
    Wallet(ErrorWallet),

    FailThread,
    ErrorBlock(String),
    TerminalReadFail,
}

impl Debug for ErrorExecution {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ErrorExecution::Initialization(error_initialization) => {
                write!(f, "{:?}", error_initialization)
            }
            ErrorExecution::Log(error_log) => write!(f, "{:?}", error_log),
            ErrorExecution::Configuration(error_configuration) => {
                write!(f, "{:?}", error_configuration)
            }
            ErrorExecution::Connection(error_connection) => write!(f, "{:?}", error_connection),
            ErrorExecution::Serialization(error_serialization) => {
                write!(f, "{:?}", error_serialization)
            }
            ErrorExecution::Message(error_message) => write!(f, "{:?}", error_message),
            ErrorExecution::Block(error_block) => write!(f, "{:?}", error_block),
            ErrorExecution::Node(error_node) => write!(f, "{:?}", error_node),
            ErrorExecution::Wallet(error_wallet) => write!(f, "{:?}", error_wallet),
            ErrorExecution::FailThread => write!(f, "ErrorFailThread"),
            ErrorExecution::ErrorBlock(error) => write!(f, "Error with block: {}", error),
            ErrorExecution::TerminalReadFail => write!(f, "ErrorTerminalReadFail"),
        }
    }
}

impl From<ErrorInitialization> for ErrorExecution {
    fn from(value: ErrorInitialization) -> Self {
        ErrorExecution::Initialization(value)
    }
}

impl From<ErrorLog> for ErrorExecution {
    fn from(value: ErrorLog) -> Self {
        ErrorExecution::Log(value)
    }
}

impl From<ErrorConfiguration> for ErrorExecution {
    fn from(value: ErrorConfiguration) -> Self {
        ErrorExecution::Configuration(value)
    }
}

impl From<ErrorConnection> for ErrorExecution {
    fn from(value: ErrorConnection) -> Self {
        ErrorExecution::Connection(value)
    }
}

impl From<ErrorMessage> for ErrorExecution {
    fn from(value: ErrorMessage) -> Self {
        ErrorExecution::Message(value)
    }
}

impl From<ErrorBlock> for ErrorExecution {
    fn from(value: ErrorBlock) -> Self {
        ErrorExecution::Block(value)
    }
}

impl From<ErrorNode> for ErrorExecution {
    fn from(value: ErrorNode) -> Self {
        ErrorExecution::Node(value)
    }
}

impl From<ErrorSerialization> for ErrorExecution {
    fn from(value: ErrorSerialization) -> Self {
        ErrorExecution::Serialization(value)
    }
}

impl From<ErrorWallet> for ErrorExecution {
    fn from(value: ErrorWallet) -> Self {
        ErrorExecution::Wallet(value)
    }
}
