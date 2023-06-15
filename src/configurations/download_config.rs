use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

const TIMESTAMP: &str = "timestamp";

#[derive(Debug, PartialEq, Clone)]
pub struct DownloadConfig {
    pub timestamp: u32,
}

impl Parsable for DownloadConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(DownloadConfig {
            timestamp: u32::parse(TIMESTAMP, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_CONNECTION: DownloadConfig = DownloadConfig { timestamp: 0 };

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "download {
            timestamp = 0
        }";

        let name = "download";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = DownloadConfig::parse(name, &map);

        assert_eq!(Ok(CONFIG_CONNECTION), connection_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "download {
            timestamp =      0
        }";

        let name = "download";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = DownloadConfig::parse(name, &map);

        assert_eq!(Ok(CONFIG_CONNECTION), connection_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let configuration = "download {
        }";

        let name = "download";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = DownloadConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), connection_result);
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let configuration = "download {
            timestamp = 0
            timestamp = 0
        }";

        let name = "download";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = DownloadConfig::parse(name, &map);

        assert_eq!(Ok(CONFIG_CONNECTION), connection_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";

        let name = "download";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = DownloadConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), connection_result);
    }
}
