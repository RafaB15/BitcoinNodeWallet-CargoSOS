use super::estructura_deserializable::EstructuraDeserializable;
use super::deserializable::deserializar;
use super::parse_error::ErroresParseo;
use std::collections::HashMap;

const LOGS_CONFIG: &str = "Logs";

const FILEPATH_LOG: &str = "filepath_log";

#[derive(Debug, std::cmp::PartialEq)]
pub struct LogConfig {
    ///La ruta al archivo en donde vamos a escribir el log
    pub filepath_log: String,
}

impl<'d> EstructuraDeserializable<'d> for LogConfig {
    type Valor = LogConfig;

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ErroresParseo> {
        Ok(LogConfig { 
            filepath_log: deserializar::<String>(FILEPATH_LOG, &settings_dictionary)? 
        })
    }

    fn nombre() -> String {
        LOGS_CONFIG.to_string()
    }
}
