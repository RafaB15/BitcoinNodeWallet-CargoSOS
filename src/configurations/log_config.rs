use super::deserializable::deserialize;
use super::deserializable_structure::DeserializeStructure;
use super::parse_error::ParseError;
use std::collections::HashMap;

const FILEPATH_LOG: &str = "filepath_log";

/// Configuration for the logs process
#[derive(Debug, std::cmp::PartialEq)]
pub struct LogConfig {
    /// The file path to where to write the logs message
    pub filepath_log: String,
}

impl<'d> DeserializeStructure<'d> for LogConfig {
    type Value = LogConfig;

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ParseError> {
        Ok(LogConfig {
            filepath_log: deserialize::<String>(FILEPATH_LOG, &settings_dictionary)?,
        })
    }
}
