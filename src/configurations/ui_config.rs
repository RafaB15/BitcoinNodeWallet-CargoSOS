use super::{
    error_configuration::ErrorConfiguration,
    interface::Interface,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use std::cmp::PartialEq;

const INTERFACE: &str = "interface";

/// It represents all the data needed for the UI
#[derive(Debug, PartialEq, Clone)]
pub struct UIConfig {
    /// It's which interface will be used
    pub interface: Interface,
}

impl Parsable for UIConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(UIConfig {
            interface: Interface::parse(INTERFACE, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "UI {
            interface = GUI
        }";
        let name = "UI";
        let map = parse_structure(configuration.to_string()).unwrap();

        let ui_result = UIConfig::parse(name, &map);

        let ui_log = UIConfig {
            interface: Interface::Gui,
        };

        assert_eq!(Ok(ui_log), ui_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "UI {
            interface =             GUI
        }";
        let name = "UI";
        let map = parse_structure(configuration.to_string()).unwrap();

        let ui_result = UIConfig::parse(name, &map);

        let ui_log = UIConfig {
            interface: Interface::Gui,
        };

        assert_eq!(Ok(ui_log), ui_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let configuration = "UI {
        }";
        let name = "UI";
        let map = parse_structure(configuration.to_string()).unwrap();

        let ui_result = UIConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), ui_result);
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let configuration = "UI {
            interface = GUI
            interface = GUI
        }";
        let name = "UI";
        let map = parse_structure(configuration.to_string()).unwrap();

        let ui_result = UIConfig::parse(name, &map);

        let ui_log = UIConfig {
            interface: Interface::Gui,
        };

        assert_eq!(Ok(ui_log), ui_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";
        let name = "UI";
        let map = parse_structure(configuration.to_string()).unwrap();

        let ui_result = UIConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), ui_result);
    }
}
