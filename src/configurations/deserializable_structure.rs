use super::parse_error::ParseError;
use std::collections::HashMap;

pub trait DeserializeStructure<'d> {
    type Value;

    fn deserializar(
        estructura_dictionary: &'d HashMap<String, Vec<String>>,
    ) -> Result<Self::Value, ParseError> {
        let nombre = format!("[{}]", Self::name());

        if let Some(valor) = estructura_dictionary.get(nombre.as_str()) {
            let settings_dictionary = create_property_value_dictionary(valor)?;
            Ok(Self::new(settings_dictionary)?)
        } else {
            Err(ParseError::ErrorIncompleteConfiguration)
        }
    }

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self::Value, ParseError>;

    fn name() -> String {
        stringify!(self).to_string()
    }
}

fn create_property_value_dictionary(
    text: &Vec<String>,
) -> Result<HashMap<String, String>, ParseError> {
    let mut config_dictionary: HashMap<String, String> = HashMap::new();

    for line in text {
        if !line.contains(':') {
            continue;
        }

        let (key, value) = slit_linea(line)?;

        if config_dictionary.contains_key(&key) {
            return Err(ParseError::ErrorEncounterFieldMoreThanOnes);
        }

        config_dictionary.insert(key, value);
    }

    Ok(config_dictionary)
}

fn slit_linea(text_line: &str) -> Result<(String, String), ParseError> {
    let mut split_line = text_line.split(':');

    match (split_line.next(), split_line.next()) {
        (Some(key), Some(value)) => Ok((key.trim().to_string(), value.trim().to_string())),
        _ => Err(ParseError::ErrorInvalidFormat),
    }
}
