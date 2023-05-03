use crate::errors::parse_error::ErroresParseo;
use std::collections::HashMap;

pub trait EstructuraDeserializable<'d> {
    type Valor;

    fn deserializar(&self, estructura_dictionary: &'d HashMap<String, Vec<String>>) -> Result<Self::Valor, ErroresParseo> {
        let nombre = format!("[{}]", Self::nombre());

        if let Some(valor) = estructura_dictionary.get(nombre.as_str()) {
            let settings_dictionary = create_property_value_dictionary(valor)?;
            Ok(Self::new(settings_dictionary)?)
        } else {
            Err(ErroresParseo::ErrorConfiguracionIncompleta)
        }
    }

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self::Valor, ErroresParseo>;

    fn nombre() -> String {
        stringify!(self).to_string()
    }
}

fn create_property_value_dictionary(
    file_files: &Vec<String>,
) -> Result<HashMap<String, String>, ErroresParseo> {
    let mut config_dictionary: HashMap<String, String> = HashMap::new();

    for line in file_files {
        let (key, value) = slit_linea(line)?;

        if config_dictionary.contains_key(&key) {
            return Err(ErroresParseo::ErrorCategoriaAparareceMasDeUnaVez);
        }

        config_dictionary.insert(key, value);
    }

    Ok(config_dictionary)
}

fn slit_linea(file_line: &String) -> Result<(String, String), ErroresParseo> {
    let mut split_line = file_line.split(':');

    let (key, value) = match (split_line.next(), split_line.next()) {
        (Some(key), Some(value)) => (key, value),
        _ => {
            return Err(ErroresParseo::ErrorFormatoIncorrecto);
        }
    };

    Ok((key.trim().to_string(), value.trim().to_string()))
}