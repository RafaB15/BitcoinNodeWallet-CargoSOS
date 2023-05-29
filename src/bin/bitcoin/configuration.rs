use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig,
    error_configuration::ErrorConfiguration,
    log_config::LogConfig,
    parsable::{parse_structure, Parsable},
};

use std::io::Read;

const CONNECTION_CONFIG: &str = "Connection";
const LOGS_CONFIG: &str = "Logs";

pub struct Configuration {
    pub log_config: LogConfig,
    pub connection_config: ConnectionConfig,
}

impl Configuration {
    pub fn new<R: Read>(mut stream: R) -> Result<Self, ErrorConfiguration> {
        let mut value = String::new();
        if stream.read_to_string(&mut value).is_err() {
            return Err(ErrorConfiguration::ErrorReadableError);
        }

        let map = parse_structure(value)?;
        Ok(Configuration {
            log_config: LogConfig::parse(LOGS_CONFIG, &map)?,
            connection_config: ConnectionConfig::parse(CONNECTION_CONFIG, &map)?,
        })
    }

    pub fn separate(self) -> (LogConfig, ConnectionConfig) {
        (self.log_config, self.connection_config)
    }
}
