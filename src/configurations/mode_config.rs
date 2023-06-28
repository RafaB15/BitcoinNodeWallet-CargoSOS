use super::{client_config::ClientConfig, server_config::ServerConfig};

use std::cmp::PartialEq;

/// Configuration for the Mode process
#[derive(Debug, PartialEq, Clone)]
pub enum ModeConfig {
    /// Server if mode config contains server information
    Server(ServerConfig),

    /// Client if mode config contains client information
    Client(ClientConfig),
}
