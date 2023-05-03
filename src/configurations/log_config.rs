use crate::errors::parse_error::ErroresParseo;
use std::collections::HashMap;
use super::deserializable::Deserializable;

#[derive(Debug)]
pub struct LogConfig {
    ///La ruta al archivo en donde vamos a escribir el log
    pub filepath_log: String,
}

impl LogConfig {
    pub fn new<'d>(settings_dictionary: &'d HashMap<String, String>) -> Result<Self, ErroresParseo> {
        let mut log_config: LogConfig;

        log_config.filepath_log.deserializar(settings_dictionary)?;

        Ok(log_config)
    }
}

