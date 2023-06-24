use crate::configurations::{
    error_configuration::ErrorConfiguration,
    parsable::{value_from_map, KeyValueMap, Parsable},
};

use std::{cmp::PartialEq, str::FromStr};

/// It's the representation of Initial Block Download method
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IBDMethod {
    BlocksFirst,
    HeaderFirst,
}

impl FromStr for IBDMethod {
    type Err = ErrorConfiguration;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "BlocksFirst" => Ok(IBDMethod::BlocksFirst),
            "HeaderFirst" => Ok(IBDMethod::HeaderFirst),
            _ => Err(ErrorConfiguration::ErrorCantParseValue(format!(
                "ibd method of {:?}",
                value
            ))),
        }
    }
}

impl Parsable for IBDMethod {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let value = value_from_map(name.to_string(), map)?;
        value.parse::<IBDMethod>()
    }
}
