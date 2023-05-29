use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use crate::connections::{
    dns_seeder::DNSSeeder, ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P,
};

use crate::messages::bitfield_services::BitfieldServices;

use std::cmp::PartialEq;

const DNS_SEEDER: &str = "dns_seeder";
const P2P_PROTOCOL_VERSION: &str = "p2p_protocol_version";
const IBD_METHOD: &str = "ibd_method";
const PEER_COUNT_MAX: &str = "peer_count_max";
const BLOCK_HEIGHT: &str = "block_height";
const SERVICES: &str = "services";

#[derive(Debug, PartialEq, Clone)]
pub struct ConnectionConfig {
    pub dns_seeder: DNSSeeder,
    /// Es la versión del protocolo peer to peer que se planea utilizar
    pub p2p_protocol_version: ProtocolVersionP2P,
    ///El método usado para el initial blocks download
    pub ibd_method: IBDMethod,

    pub peer_count_max: usize,

    pub block_height: i32,

    pub services: BitfieldServices,
}

impl Parsable for ConnectionConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        println!("{:?}", map);

        println!("{DNS_SEEDER}");
        let dns_seeder = DNSSeeder::parse(DNS_SEEDER, &map)?;
        println!("{P2P_PROTOCOL_VERSION}");
        let p2p_protocol_version = ProtocolVersionP2P::parse(P2P_PROTOCOL_VERSION, &map)?;
        println!("{IBD_METHOD}");
        let ibd_method =IBDMethod::parse(IBD_METHOD, &map)?;
        println!("{PEER_COUNT_MAX}");
        let peer_count_max = usize::parse(PEER_COUNT_MAX, &map)?;
        println!("{BLOCK_HEIGHT}");
        let block_height = i32::parse(BLOCK_HEIGHT, &map)?;
        println!("{SERVICES}");
        let services = BitfieldServices::parse(SERVICES, &map)?;

        Ok(ConnectionConfig {
            dns_seeder,
            p2p_protocol_version,
            ibd_method,
            peer_count_max,
            block_height,
            services,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::connections::suppored_services::SupportedServices;

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "connection {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                port = 18333
            }
            p2p_protocol_version = V70015
            ibd_method = HeaderFirst
            peer_count_max = 8
            block_height = 0
            services = [Unname]
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();
        println!("{:?}", map);

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            dns_seeder: DNSSeeder {
                dns_addr: "seed.testnet.bitcoin.sprovoost.nl".to_string(),
                port_number: 18333,
            },
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            peer_count_max: 8,
            block_height: 0,
            services: BitfieldServices {
                elements: vec![SupportedServices::Unname],
            },
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "connection {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                             port = 18333
        }
                      p2p_protocol_version = V70015
            ibd_method=                                 HeaderFirst
            peer_count_max = 8
            block_height = 0
            services = [Unname]
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            dns_seeder: DNSSeeder {
                dns_addr: "seed.testnet.bitcoin.sprovoost.nl".to_string(),
                port_number: 18333,
            },
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            peer_count_max: 8,
            block_height: 0,
            services: BitfieldServices {
                elements: vec![SupportedServices::Unname],
            },
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let configuration = "connection {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                port = 18333
            }
            p2p_protocol_version = V70015
            peer_count_max = 8
            block_height = 0
            services = [Unname]
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        assert_eq!(
            Err(ErrorConfiguration::ValueNotFound),
            connection_result
        );
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let configuration = "connection {
            dns_seeder {
                seed = seed.testnet.bitcoin.sprovoost.nl
                port = 18333
            }
            p2p_protocol_version = V70015
            ibd_method = HeaderFirst
            ibd_method = HeaderFirst
            peer_count_max = 8
            block_height = 0
            services = [Unname]
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            dns_seeder: DNSSeeder {
                dns_addr: "seed.testnet.bitcoin.sprovoost.nl".to_string(),
                port_number: 18333,
            },
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            peer_count_max: 8,
            block_height: 0,
            services: BitfieldServices {
                elements: vec![SupportedServices::Unname],
            },
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        assert_eq!(
            Err(ErrorConfiguration::ValueNotFound),
            connection_result
        );
    }
}
