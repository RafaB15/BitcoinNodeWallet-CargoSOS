use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use crate::connections::dns_seeder::DNSSeeder;

use std::net::Ipv4Addr;
use std::cmp::PartialEq;

const DNS_SEEDER: &str = "dns_seeder";
const PEER_COUNT_MAX: &str = "peer_count_max";
const CLIENT_COUNT_MAX: &str = "client_count_max";
const PORT: &str = "port";
const ADDRESS: &str = "address";

/// Configuration for the server process
#[derive(Debug, PartialEq, Clone)]
pub struct ServerConfig {
    /// It's the DNS from where the potential peers will be obtaineds
    pub dns_seeder: DNSSeeder,

    /// It's the maximum number of peers that will be connected
    pub peer_count_max: usize,

    /// It's the maximum number of clients that will be connected
    pub client_count_max: usize,

    /// It's the port number where the server will be listening
    pub own_port: u16,

    /// It's the address where the server will be listening
    pub address: Ipv4Addr,
}

impl Parsable for ServerConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(ServerConfig {
            dns_seeder: DNSSeeder::parse(DNS_SEEDER, &map)?,
            peer_count_max: usize::parse(PEER_COUNT_MAX, &map)?,
            client_count_max: usize::parse(CLIENT_COUNT_MAX, &map)?,
            own_port: u16::parse(PORT, &map)?,
            address: Ipv4Addr::parse(ADDRESS, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_accept_valid_input() {
        let server = "server {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                port = 18333
            }
            peer_count_max = 8
            client_count_max = 8
            own_port = 18333
            address = 127.0.0.1
        }";

        let name = "server";
        let map = parse_structure(server.to_string()).unwrap();

        let server_result = ServerConfig::parse(name, &map);

        let config_server = ServerConfig {
            dns_seeder: DNSSeeder::new("seed.testnet.bitcoin.sprovoost.nl", 18333),
            peer_count_max: 8,
            client_count_max: 8,
            own_port: 18333,
            address: Ipv4Addr::new(127, 0, 0, 1),
        };

        assert_eq!(Ok(config_server), server_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let server = "server {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                             port = 18333
        }
                      peer_count_max = 8
            client_count_max=                                 8
            own_port = 18333
            address = 127.0.0.1
        }";

        let name = "server";
        let map = parse_structure(server.to_string()).unwrap();

        let server_result = ServerConfig::parse(name, &map);

        let server_config = ServerConfig {
            dns_seeder: DNSSeeder::new("seed.testnet.bitcoin.sprovoost.nl", 18333),
            peer_count_max: 8,
            client_count_max: 8,
            own_port: 18333,
            address: Ipv4Addr::new(127, 0, 0, 1),
        };

        assert_eq!(Ok(server_config), server_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let server = "server {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                port = 18333
            }
            peer_count_max = 8
            own_port = 18333
            address = 127.0.0.1
        }";

        let name = "server";
        let map = parse_structure(server.to_string()).unwrap();

        let server_result = ServerConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), server_result);
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let server = "server {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                port = 18333
            }
            peer_count_max = 8
            peer_count_max = 8
            client_count_max = 8
            own_port = 18333
            address = 127.0.0.1
        }";

        let name = "server";
        let map = parse_structure(server.to_string()).unwrap();

        let server_result = ServerConfig::parse(name, &map);

        let server_config = ServerConfig {
            dns_seeder: DNSSeeder::new("seed.testnet.bitcoin.sprovoost.nl", 18333),
            peer_count_max: 8,
            client_count_max: 8,
            own_port: 18333,
            address: Ipv4Addr::new(127, 0, 0, 1),
        };

        assert_eq!(Ok(server_config), server_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";

        let name = "server";
        let map = parse_structure(configuration.to_string()).unwrap();

        let server_result = ServerConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), server_result);
    }
}

