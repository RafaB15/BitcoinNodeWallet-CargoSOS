#[derive(Debug)]
pub enum Level {
    NODE,
    WALLET,
    TRANSACTION, 
    CONFIGURATION,
    CONNECTION,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::NODE => write!(f, "NODE"),
            Level::WALLET => write!(f, "WALLET"),
            Level::TRANSACTION => write!(f, "TRANSACTION"),
            Level::CONFIGURATION => write!(f, "CONFIGURATION"),
            Level::CONNECTION => write!(f, "CONNECTION"),
        }
    }
}