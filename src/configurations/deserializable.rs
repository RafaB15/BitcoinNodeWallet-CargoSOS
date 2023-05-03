use super::parse_error::ErroresParseo;
use std::collections::HashMap;
use std::str::FromStr;

pub fn deserializar<V : FromStr>(nombre: &str, settings_dictionary: &HashMap<String, String>) -> Result<V, ErroresParseo> {
    if let Some(valor) = settings_dictionary.get(nombre) {
        match valor.parse::<V>() {
            Ok(resultado) => Ok(resultado),
            _ => return Err(ErroresParseo::ErrorConfiguracionIncompleta),
        }
    } else {
        Err(ErroresParseo::ErrorConfiguracionIncompleta)
    }
}