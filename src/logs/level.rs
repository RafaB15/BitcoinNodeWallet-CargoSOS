#[derive(Debug, Clone)]
pub enum Level {
    NODE,
    WALLET,
    TRANSACTION,
    CONFIGURATION,
    CONNECTION,
    FILE,
    INTERFACE,
    ERROR,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::NODE => write!(f, "NODE"),
            Level::WALLET => write!(f, "WALLET"),
            Level::TRANSACTION => write!(f, "TRANSACTION"),
            Level::CONFIGURATION => write!(f, "CONFIGURATION"),
            Level::CONNECTION => write!(f, "CONNECTION"),
            Level::FILE => write!(f, "FILE"),
            Level::INTERFACE => write!(f, "INTERFACE"),
            Level::ERROR => write!(f, "ERROR"),
        }
    }
}
