use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use std::cmp::PartialEq;

const FILEPATH_LOG: &str = "filepath_log";
const SHOW_CONSOLE: &str = "show_console";

/// Configuration for the logs process
#[derive(Debug, PartialEq, Clone)]
pub struct LogConfig {
    /// The file path to where to write the logs message
    pub filepath_log: String,

    pub show_console: bool,
}

impl Parsable for LogConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(LogConfig {
            filepath_log: String::parse(FILEPATH_LOG, &map)?,
            show_console: bool::parse(SHOW_CONSOLE, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "logs {
            filepath_log = log_test.txt
            show_console = true
        }";
        let name = "logs";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = LogConfig::parse(name, &map);

        let config_log = LogConfig {
            filepath_log: "log_test.txt".to_string(),
            show_console: true,
        };

        assert_eq!(Ok(config_log), log_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "logs {
            filepath_log =                               log_test.txt
            show_console = true
        }";
        let name = "logs";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = LogConfig::parse(name, &map);

        let config_log = LogConfig {
            filepath_log: "log_test.txt".to_string(),
            show_console: true,
        };

        assert_eq!(Ok(config_log), log_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let configuration = "logs {
            show_console = true
        }";
        let name = "logs";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = LogConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), log_result);
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let configuration = "logs {
            filepath_log = log_test.txt
            filepath_log = log_test.txt
            show_console = true
        }";
        let name = "logs";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = LogConfig::parse(name, &map);

        let config_log = LogConfig {
            filepath_log: "log_test.txt".to_string(),
            show_console: true,
        };

        assert_eq!(Ok(config_log), log_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";
        let name = "logs";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = LogConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), log_result);
    }
}
