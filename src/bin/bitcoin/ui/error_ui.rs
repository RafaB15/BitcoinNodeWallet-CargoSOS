use crate::process::error_process::ErrorProcess;

use std::convert::From;

/// It represents all posible errors that can occur in the TUI
#[derive(Debug, Clone)]
pub enum ErrorUI {
    /// It will appear when the user selects an invalid option from the menu
    InvalidMenuOption,

    /// It will appear when the terminal read fails
    TerminalReadFail,

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

    /// It will appear when can't create the default value
    CannotCreateDefault,

    /// It will appear when try to get a value that is already loaded
    AlreadyLoaded,

    /// It will appear when we try to unwrap an Arc
    CannotUnwrapArc,

    /// It will appear when we try to get the inner value of a mutex
    CannotGetInner,

    /// It will appear when we try to send a signal to the front and it fails
    FailedSignalToFront(String),

    /// It will appear when an element of the front is missing
    MissingElement(String),

    /// It will appear when a receiver is missing
    MissingReceiver,
}

impl From<ErrorProcess> for ErrorUI {
    fn from(value: ErrorProcess) -> Self {
        match value {
            ErrorProcess::ErrorReading => {
                ErrorUI::ErrorReading("While processing data".to_string())
            }
            ErrorProcess::ErrorWriting => {
                ErrorUI::ErrorWriting("While processing data".to_string())
            }
            ErrorProcess::FailThread => ErrorUI::FailThread("While processing data".to_string()),
            ErrorProcess::ErrorFromPeer(message) => ErrorUI::ErrorFromPeer(message),
            ErrorProcess::CannotCreateDefault => ErrorUI::CannotCreateDefault,
            ErrorProcess::AlreadyLoaded => ErrorUI::AlreadyLoaded,
            ErrorProcess::CannotGetInner => ErrorUI::CannotGetInner,
            ErrorProcess::CannotUnwrapArc => ErrorUI::CannotUnwrapArc,
            ErrorProcess::TransactionWithoutSufficientFunds => {
                ErrorUI::TransactionWithoutSufficientFunds
            }
            ErrorProcess::TransactionCreationFail => ErrorUI::TransactionCreationFail,
        }
    }
}
