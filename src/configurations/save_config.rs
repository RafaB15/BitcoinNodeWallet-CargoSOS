use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use std::cmp::PartialEq;

const READ_BLOCK_CHAIN: &str = "read_block_chain";
const WRITE_BLOCK_CHAIN: &str = "write_block_chain";

const READ_WALLET: &str = "read_wallet";
const WRITE_WALLET: &str = "write_wallet";

/// It represents all the data needed to load and save the data of the program
#[derive(Debug, PartialEq, Clone)]
pub struct SaveConfig {
    /// It's the file name where the block chain will be loaded
    pub read_block_chain: Option<String>,

    /// It's the file name where the wallet will be loaded
    pub read_wallet: Option<String>,

    /// It's the file name where the block chain will be saved
    pub write_block_chain: Option<String>,

    /// It's the file name where the wallet will be saved
    pub write_wallet: Option<String>,
}

impl Parsable for SaveConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(SaveConfig {
            read_block_chain: Option::<String>::parse(READ_BLOCK_CHAIN, &map)?,
            write_block_chain: Option::<String>::parse(WRITE_BLOCK_CHAIN, &map)?,
            read_wallet: Option::<String>::parse(READ_WALLET, &map)?,
            write_wallet: Option::<String>::parse(WRITE_WALLET, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "save {
            read_block_chain = save_test.txt
            write_block_chain = save_test2.txt
            write_wallet = save_w_test2.txt
            read_wallet = save_w_test.txt
        }";
        let name = "save";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = SaveConfig::parse(name, &map);

        let config_save = SaveConfig {
            read_block_chain: Some("save_test.txt".to_string()),
            write_block_chain: Some("save_test2.txt".to_string()),
            read_wallet: Some("save_w_test.txt".to_string()),
            write_wallet: Some("save_w_test2.txt".to_string()),
        };

        assert_eq!(Ok(config_save), log_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "save {
            read_block_chain =                 save_test.txt
            write_block_chain            = save_test2.txt
            write_wallet = save_w_test2.txt
            read_wallet = save_w_test.txt
        }";
        let name = "save";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = SaveConfig::parse(name, &map);

        let config_save = SaveConfig {
            read_block_chain: Some("save_test.txt".to_string()),
            write_block_chain: Some("save_test2.txt".to_string()),
            read_wallet: Some("save_w_test.txt".to_string()),
            write_wallet: Some("save_w_test2.txt".to_string()),
        };

        assert_eq!(Ok(config_save), log_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let configuration = "save {
            read_block_chain = save_test.txt
            write_wallet = save_w_test2.txt
            read_wallet = save_w_test.txt
        }";
        let name = "save";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = SaveConfig::parse(name, &map);

        let config_missing = SaveConfig {
            read_block_chain: Some("save_test.txt".to_string()),
            write_block_chain: None,
            read_wallet: Some("save_w_test.txt".to_string()),
            write_wallet: Some("save_w_test2.txt".to_string()),
        };

        assert_eq!(Ok(config_missing), log_result);
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let configuration = "save {
            read_block_chain = save_test.txt
            write_block_chain = save_test2.txt
            write_block_chain = save_test2.txt
            write_wallet = save_w_test2.txt
            read_wallet = save_w_test.txt
        }";
        let name = "save";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = SaveConfig::parse(name, &map);

        let config_save = SaveConfig {
            read_block_chain: Some("save_test.txt".to_string()),
            write_block_chain: Some("save_test2.txt".to_string()),
            read_wallet: Some("save_w_test.txt".to_string()),
            write_wallet: Some("save_w_test2.txt".to_string()),
        };

        assert_eq!(Ok(config_save), log_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";
        let name = "save";
        let map = parse_structure(configuration.to_string()).unwrap();

        let log_result = SaveConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), log_result);
    }
}
