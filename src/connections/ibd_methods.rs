use crate::configurations::{
    error_configuration::ErrorConfiguration,
    parsable::{value_from_map, KeyValueMap, Parsable},
};

use std::{cmp::PartialEq, str::FromStr};

const BLOCKS_FIRST: &str = "BlocksFirst";
const HEADER_FIRST: &str = "HeaderFirst";

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
            BLOCKS_FIRST => Ok(IBDMethod::BlocksFirst),
            HEADER_FIRST => Ok(IBDMethod::HeaderFirst),
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

#[cfg(test)]
mod tests {

    use super::*;

    use crate::configurations::parsable::parse_structure;

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "ibd_methods = BlocksFirst";

        let name = "ibd_methods";
        let map = parse_structure(configuration.to_string()).unwrap();

        let ibd_methods_result = IBDMethod::parse(name, &map);

        let expected_ibd_methods = IBDMethod::BlocksFirst;

        assert_eq!(Ok(expected_ibd_methods), ibd_methods_result);
    }
}