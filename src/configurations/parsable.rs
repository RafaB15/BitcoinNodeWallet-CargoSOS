use super::error_configuration::ErrorConfiguration;

use std::{collections::HashMap, str::FromStr};

pub type Key = String;
pub type Value = String;

pub type KeyValueMap = HashMap<Key, Value>;

pub trait Parsable
where
    Self: Sized,
{
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration>;
}

pub fn parse_structure(value: Value) -> Result<KeyValueMap, ErrorConfiguration> {
    let mut map: KeyValueMap = HashMap::new();

    let mut extend: bool = false;
    let text: Vec<String> = value
        .split(|character| match extend {
            true => character == '}',
            false => {
                if character == '{' {
                    extend = true;
                }
                character == '=' || character == '\n'
            }
        })
        .map(|valor| valor.to_string().replace('{', ""))
        .collect();

    for (i, key) in text.iter().enumerate().step_by(2) {
        let value = match text.get(i + 1) {
            Some(value) => value,
            None => continue,
        };

        map.insert(key.trim().to_string(), value.trim().to_string());
    }

    Ok(map)
}

pub fn value_from_map(key: Key, map: &KeyValueMap) -> Result<Value, ErrorConfiguration> {
    match map.get(&key) {
        Some(value) => Ok(value.clone()),
        None => Err(ErrorConfiguration::ErrorReadableError),
    }
}

impl<V> Parsable for V
where
    V: FromStr,
{
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let value = value_from_map(name.to_string(), map)?;
        match value.parse::<V>() {
            Ok(parse_value) => Ok(parse_value),
            _ => Err(ErrorConfiguration::ErrorCantParseValue),
        }
    }
}
