use crate::errors::parse_error::ErroresParseo;
use std::collections::HashMap;

const FILEPATH_LOG: &str = "filepath_log";

#[derive(Debug)]
pub struct LogConfig {
    ///La ruta al archivo en donde vamos a escribir el log
    pub filepath_log: String,
}

impl LogConfig {
    pub fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ErroresParseo> {
        let mut filepath_log: Option<String> = None;

        for (key, value) in settings_dictionary {
            match key.as_str() {
                FILEPATH_LOG => filepath_log = Some(value),
                _ => {
                    return Err(ErroresParseo::ParseoValorNoReconocido);
                }
            }
        }

        if let Some(filepath_log) = filepath_log {
            Ok(LogConfig { filepath_log })
        } else {
            Err(ErroresParseo::NoSuficientesValores)
        }
    }
}
