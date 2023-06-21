use super::{
    error_initialization::ErrorInitialization, gui::error_gui::ErrorGUI,
    process::error_process::ErrorProcess, tui::error_tui::ErrorTUI,
};

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

/// It represents all posible errors that can occur in the execution of the program
pub enum ErrorExecution {
    /// It represents all posible errors that can occur initializing the program
    Initialization(ErrorInitialization),

    /// It represents all posible errors that can occur in the logs
    Log(ErrorLog),

    ///
    Configuration(ErrorConfiguration),

    ///
    Connection(ErrorConnection),

    /// It represents all posible errors that can occur in the process of serializing and deserializing
    Serialization(ErrorSerialization),

    ///
    Message(ErrorMessage),

    ///
    Block(ErrorBlock),

    /// It represents all posible errors that can occur while making the protocols of a node
    Node(ErrorNode),

    ///
    Wallet(ErrorWallet),

    /// It represents all posible errors that can occur in the process of connecting with a peer
    Process(ErrorProcess),

    ///
    GUI(ErrorGUI),

    /// It represents all posible errors that can occur in the TUI
    TUI(ErrorTUI),

    /// It will appear when a thread panics and fails
    FailThread,
    _ErrorBlock(String),
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
            ErrorExecution::Process(error_process) => write!(f, "{:?}", error_process),
            ErrorExecution::GUI(error_gui) => write!(f, "{:?}", error_gui),
            ErrorExecution::TUI(error_tui) => write!(f, "{:?}", error_tui),
            ErrorExecution::FailThread => write!(f, "ErrorFailThread"),
            ErrorExecution::_ErrorBlock(error) => write!(f, "Error with block: {}", error),
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

impl From<ErrorGUI> for ErrorExecution {
    fn from(value: ErrorGUI) -> Self {
        ErrorExecution::GUI(value)
    }
}

impl From<ErrorTUI> for ErrorExecution {
    fn from(value: ErrorTUI) -> Self {
        ErrorExecution::TUI(value)
    }
}

impl From<ErrorProcess> for ErrorExecution {
    fn from(value: ErrorProcess) -> Self {
        ErrorExecution::Process(value)
    }
}
