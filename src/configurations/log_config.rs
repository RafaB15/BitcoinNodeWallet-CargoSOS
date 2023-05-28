use super::{
    parsable::{
        Parsable,
        KeyValueMap,
        value_from_map,
        parse_structure,
    },
    error_configuration::ErrorConfiguration,
};

use std::{
    cmp::PartialEq,
};

const FILEPATH_LOG: &str = "filepath_log";

/// Configuration for the logs process
#[derive(Debug, PartialEq)]
pub struct LogConfig {
    /// The file path to where to write the logs message
    pub filepath_log: String,
}

impl Parsable for LogConfig {

    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(LogConfig {
            filepath_log: String::parse(FILEPATH_LOG, &map)?,
        })
    }
}