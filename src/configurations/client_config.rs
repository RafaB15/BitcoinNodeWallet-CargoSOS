use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use crate::connections::dns_seeder::DNSSeeder;

use std::net::ipv4;
use std::cmp::PartialEq;

const PORT: &str = "port";
const ADDRESS: &str = "address";

/// Configuration for the client process
#[derive(Debug, PartialEq, Clone)]
pub struct ClientConfig {

    /// It's the port number where the client will be connected to
    pub port: u16,

    /// It's the address where the client will be connected to
    pub address: ipv4::Ipv4Addr,
}

impl Parsable for ClientConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(ClientConfig {
            port: u16::parse(PORT, &map)?,
            address: ipv4::Ipv4Addr::parse(ADDRESS, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_accept_valid_input() {
        let client = "client {
            port = 18333
            address = 127.0.0.1
        }";

        let name = "client";
        let map = parse_structure(client.to_string()).unwrap();

        let client_result = ClientConfig::parse(name, &map);

        let config_client = ClientConfig {
            port: 18333,
            address: ipv4::Ipv4Addr::new(127, 0, 0, 1),
        };

        assert_eq!(Ok(config_client), client_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "client {
                    port = 18333
            address =                                       127.0.0.1
        }";

        let name = "client";
        let map = parse_structure(server.to_string()).unwrap();

        let client_result = Client::parse(name, &map);

        let client_config = ClientConfig {
            port: 18333,
            address: ipv4::Ipv4Addr::new(127, 0, 0, 1),
        };

        assert_eq!(Ok(client_config), client_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let client = "client {
            address = 127.0.0.1
        }";

        let name = "client";
        let map = parse_structure(client.to_string()).unwrap();

        let client_result = ClientConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), client_result);
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let client = "client {
            port = 18333
            port = 18333
            address = 127.0.0.1
        }";

        let name = "client";
        let map = parse_structure(client.to_string()).unwrap();

        let client_result = ClientConfig::parse(name, &map);

        let client_config = ClientConfig {
            port: 18333,
            address: ipv4::Ipv4Addr::new(127, 0, 0, 1),
        };

        assert_eq!(Ok(client_config), client_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";

        let name = "client";
        let map = parse_structure(configuration.to_string()).unwrap();

        let client_result = ClientConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), client_result);
    }
}