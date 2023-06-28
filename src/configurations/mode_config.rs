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

///Implementación del trait que permite hacer parse
impl FromStr for Interface {
    type Err = ErrorConfiguration;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Server" => Ok(ModeConfig::Server),
            "Client" => Ok(ModeConfig::Client),
            _ => Err(ErrorConfiguration::ErrorCantParseValue(format!(
                "mode of {:?}",
                s
            ))),
        }
    }
}

impl Parsable for Interface {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let value = value_from_map(name.to_string(), map)?;
        match value.parse::<ModeConfig>() {
            Ok(value) => Ok(value),
            _ => Err(ErrorConfiguration::ErrorCantParseValue(format!(
                "mode of {:?}",
                value
            ))),
        }
    }
}

