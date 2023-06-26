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
}
