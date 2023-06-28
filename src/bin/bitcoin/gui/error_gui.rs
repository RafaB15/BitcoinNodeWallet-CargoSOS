use crate::process::error_process::ErrorProcess;

#[derive(Debug, Clone)]
pub enum ErrorGUI {
    FailedSignalToFront(String),
    ErrorWriting(String),
    ErrorReading(String),
    CannotUnwrapArc,
    ErrorFromPeer(String),
    MissingElement(String),
    CannotGetInner,
    MissingReceiver,
    ErrorInTransaction(String),
    CannotCreateDefault,
    AlreadyLoaded,
    FailThread(String),
}

impl From<ErrorProcess> for ErrorGUI {
    fn from(value: ErrorProcess) -> Self {
        match value {
            ErrorProcess::ErrorReading => {
                ErrorGUI::ErrorReading("While processing data".to_string())
            }
            ErrorProcess::ErrorWriting => {
                ErrorGUI::ErrorWriting("While processing data".to_string())
            }
            ErrorProcess::FailThread => ErrorGUI::FailThread("While processing data".to_string()),
            ErrorProcess::ErrorFromPeer(message) => ErrorGUI::ErrorFromPeer(message),
            ErrorProcess::CannotCreateDefault => ErrorGUI::CannotCreateDefault,
            ErrorProcess::AlreadyLoaded => ErrorGUI::AlreadyLoaded,
            ErrorProcess::CannotGetInner => ErrorGUI::CannotGetInner,
            ErrorProcess::CannotUnwrapArc => ErrorGUI::CannotUnwrapArc,
        }
    }
}