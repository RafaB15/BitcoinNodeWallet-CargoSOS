use crate::process::error_process::ErrorProcess;

use std::convert::From;

/// It represents all posible errors that can occur in the TUI
#[derive(Debug, Clone)]
pub enum ErrorTUI {
    /// It will appear when the user selects an invalid option from the menu
    InvalidMenuOption,

    /// It will appear when the terminal read fails
    TerminalReadFail,

    /// It will appear when we try to unwrap an Arc
    CannotUnwrapArc,

    /// It will appear when we try to get the inner value of a mutex
    CannotGetInner,

    /// It will appear when trying to create a transaction of an amount and fee greater than the balance
    TransactionWithoutSufficientFunds,

    /// It will appear when trying to create a transaction and fails to create the signature script for it
    TransactionCreationFail,

    /// It will appear when reading
    ErrorReading(String),

    /// It will appear when writing
    ErrorWriting(String),

    /// It will appear when a thread panics and fails
    FailThread(String),

    /// It will appear when a conextion with a peer fails
    ErrorFromPeer(String),
}

impl From<ErrorProcess> for ErrorTUI {
    fn from(value: ErrorProcess) -> Self {
        match value {
            ErrorProcess::ErrorReading => {
                ErrorTUI::ErrorReading("While processing data".to_string())
            }
            ErrorProcess::ErrorWriting => {
                ErrorTUI::ErrorWriting("While processing data".to_string())
            }
            ErrorProcess::FailThread => ErrorTUI::FailThread("While processing data".to_string()),
            ErrorProcess::ErrorFromPeer(message) => ErrorTUI::ErrorFromPeer(message),
        }
    }
}
