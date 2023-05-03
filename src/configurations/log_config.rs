use super::estructura_deserializable::EstructuraDeserializable;
use super::deserializable::Deserializable;
use crate::errors::parse_error::ErroresParseo;
use std::collections::HashMap;

#[derive(Debug, std::default::Default)]
pub struct LogConfig {
    ///La ruta al archivo en donde vamos a escribir el log
    pub filepath_log: String,
}

impl<'d> EstructuraDeserializable<'d> for LogConfig {
    type Valor = LogConfig;

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ErroresParseo> {
        let log_config: LogConfig = LogConfig::default();

        log_config.filepath_log.deserializar(&settings_dictionary)?;

        Ok(log_config)
    }

    fn nombre() -> String {
        "Logs".to_string()
    }
}
