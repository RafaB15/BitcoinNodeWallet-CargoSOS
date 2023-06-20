use super::error_connection::ErrorConnection;

use crate::configurations::{
    error_configuration::ErrorConfiguration,
    parsable::{value_from_map, KeyValueMap, Parsable},
};

use std::{cmp::PartialEq, str::FromStr};

#[derive(Debug, PartialEq, Clone, Copy)]
///Enum que representa el método de Initial Block Download que se va a utilizar
pub enum IBDMethod {
    BlocksFirst,
    HeaderFirst,
}
///Implementación del trait que permite hacer parse
impl FromStr for IBDMethod {
    type Err = ErrorConnection;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BlocksFirst" => Ok(IBDMethod::BlocksFirst),
            "HeaderFirst" => Ok(IBDMethod::HeaderFirst),
            _ => Err(ErrorConnection::ErrorInvalidInputParse),
        }
    }
}

impl Parsable for IBDMethod {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let value = value_from_map(name.to_string(), map)?;
        match value.parse::<IBDMethod>() {
            Ok(value) => Ok(value),
            _ => Err(ErrorConfiguration::ErrorCantParseValue(format!(
                "ibd method of {:?}",
                value
            ))),
        }
    }
}
