use super::{
    server_config::ServerConfig,
    client_config::ClientConfig,
};

use std::cmp::PartialEq;

/// Configuration for the Mode process
#[derive(Debug, PartialEq, Clone)]
pub enum ModeConfig {
    /// Server if mode config contains server information
    Server(ServerConfig),

    /// Client if mode config contains client information
    Client(ClientConfig),
}