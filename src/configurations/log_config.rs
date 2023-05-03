use super::deserializable::deserialize;
use super::deserializable_structure::DeserializeStructure;
use super::parse_error::ParseError;
use std::collections::HashMap;

const LOGS_CONFIG: &str = "Logs";

const FILEPATH_LOG: &str = "filepath_log";

#[derive(Debug, std::cmp::PartialEq)]
pub struct LogConfig {
    ///La ruta al archivo en donde vamos a escribir el log
    pub filepath_log: String,
}

impl<'d> DeserializeStructure<'d> for LogConfig {
    type Value = LogConfig;

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ParseError> {
        Ok(LogConfig {
            filepath_log: deserialize::<String>(FILEPATH_LOG, &settings_dictionary)?,
        })
    }

    fn name() -> String {
        LOGS_CONFIG.to_string()
    }
}
