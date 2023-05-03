use super::parse_error::ParseError;
use std::collections::HashMap;

/// It's a way to ensure the correct creation of a configuration structure
pub(super) trait DeserializeStructure<'d> {
    type Value;

    /// Returns the parse structure of a given property name
    /// 
    /// ### Errors
    ///  * `ErrorConfigurationNoFount`: It will appear when there isn't a structure with a given property name
    fn deserializar(
        name: &str,
        estructura_dictionary: &'d HashMap<String, Vec<String>>,
    ) -> Result<Self::Value, ParseError> {
        let nombre = format!("[{}]", name);

        if let Some(valor) = estructura_dictionary.get(nombre.as_str()) {
            let settings_dictionary = create_property_value_dictionary(valor)?;
            Ok(Self::new(settings_dictionary)?)
        } else {
            Err(ParseError::ErrorConfigurationNoFount)
        }
    }
    
    /// Creation of the structure given 
    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self::Value, ParseError>;
}

/// Returns the key-values pair for all the configuration of a given structure
/// 
/// ### Errors
///  * `ErrorEncounterFieldMoreThanOnes`: It will appear when the property name appears more than ones
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

/// Return the key-value pair of a line in the configuration
/// 
/// ### Errors
///  * `ErrorInvalidFormat`: It will appear when the line of the configuration isn't given by the format `key: value`
fn slit_linea(text_line: &str) -> Result<(String, String), ParseError> {
    let mut split_line = text_line.split(':');

    match (split_line.next(), split_line.next()) {
        (Some(key), Some(value)) => Ok((key.trim().to_string(), value.trim().to_string())),
        _ => Err(ParseError::ErrorInvalidFormat),
    }
}
