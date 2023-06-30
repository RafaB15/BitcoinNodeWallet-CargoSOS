use std::fmt::{Display, Formatter, Result};

const CLIENT: &str = "Client";
const PEER: &str = "Peer";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    Client,
    Peer,
}

impl Display for ConnectionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ConnectionType::Client => write!(f, "{CLIENT}"),
            ConnectionType::Peer => write!(f, "{PEER}"),
        }
    }
}
