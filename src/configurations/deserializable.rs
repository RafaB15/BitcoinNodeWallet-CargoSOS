use super::parse_error::ParseError;
use std::collections::HashMap;
use std::str::FromStr;

pub fn deserialize<V: FromStr>(
    name: &str,
    settings_dictionary: &HashMap<String, String>,
) -> Result<V, ParseError> {
    if let Some(value) = settings_dictionary.get(name) {
        match value.parse::<V>() {
            Ok(parse_value) => Ok(parse_value),
            _ => Err(ParseError::ErrorIncompleteConfiguration),
        }
    } else {
        Err(ParseError::ErrorIncompleteConfiguration)
    }
}
