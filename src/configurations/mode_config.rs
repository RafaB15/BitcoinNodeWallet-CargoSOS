use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
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