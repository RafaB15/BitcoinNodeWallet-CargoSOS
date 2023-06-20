use crate::process::error_process::ErrorProcess;

use std::convert::From;

#[derive(Debug, Clone)]
pub enum ErrorTUI {
    InvalidMenuOption,
    TerminalReadFail,
    CannotUnwrapArc,
    CannotGetInner,
    TransactionCreationFail,
    ErrorReading(String),
    ErrorWriting(String),
    FailThread(String),
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
