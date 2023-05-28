use super::error_configuration::ErrorConfiguration;

use std::{
    collections::HashMap,
    str::FromStr,
};

pub type Key = String;
pub type Value = String;

pub type KeyValueMap = HashMap<Key, Value>;

pub trait Parsable
    where Self : Sized
{
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration>;
}

pub fn parse_structure(value: Value) -> Result<KeyValueMap, ErrorConfiguration> {
    let mut map: KeyValueMap = HashMap::new();

    let mut lines: Vec<String> = Vec::new();
    value.split('\n').for_each(|line| lines.push(line.to_string()));

    Ok(map)
}

pub fn value_from_map(key: Key, map: &KeyValueMap) -> Result<Value, ErrorConfiguration> {
    match map.get(&key) {
        Some(value) => Ok(value.clone()),
        None => Err(ErrorConfiguration::ErrorReadableError),
    }
}

impl<V> Parsable for V 
    where V : FromStr
{    
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {

        let value = value_from_map(name.to_string(), map)?;
        match value.parse::<V>() {
            Ok(parse_value) => Ok(parse_value),
            _ => Err(ErrorConfiguration::ErrorCantParseValue),
        }
    }
}

