use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use crate::connections::{
    ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P,
};

use crate::messages::{bitfield_services::BitfieldServices, message_header::MagicType};

use std::cmp::PartialEq;

const P2P_PROTOCOL_VERSION: &str = "p2p_protocol_version";
const IBD_METHOD: &str = "ibd_method";
const BLOCK_HEIGHT: &str = "block_height";
const SERVICES: &str = "services";
const MAGIC_NUMBERS: &str = "magic_numbers";
const NONCE: &str = "nonce";
const USER_AGENT: &str = "user_agent";
const RELAY: &str = "relay";

/// It represents all the data needed to establish a connection
#[derive(Debug, PartialEq, Clone)]
pub struct ConnectionConfig {
    /// It's the version of the peer to peer protocol that will be used
    pub p2p_protocol_version: ProtocolVersionP2P,

    /// It's the method used for the initial blocks download
    pub ibd_method: IBDMethod,

    /// It's the block height from where the initial blocks download will start
    pub block_height: i32,

    /// It's the services that this node will offer
    pub services: BitfieldServices,

    /// It's the magic numbers that will be used to identify the network
    pub magic_numbers: MagicType,

    /// It's used to detect connections to self
    pub nonce: u64,

    /// It's used in the version message
    pub user_agent: String,

    /// It's the flag that indicates if the node will relay transactions
    pub relay: bool,
}

impl Parsable for ConnectionConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(ConnectionConfig {
            p2p_protocol_version: ProtocolVersionP2P::parse(P2P_PROTOCOL_VERSION, &map)?,
            ibd_method: IBDMethod::parse(IBD_METHOD, &map)?,
            block_height: i32::parse(BLOCK_HEIGHT, &map)?,
            services: BitfieldServices::parse(SERVICES, &map)?,
            magic_numbers: MagicType::parse(MAGIC_NUMBERS, &map)?,
            nonce: u64::parse(NONCE, &map)?,
            user_agent: Option::<String>::parse(USER_AGENT, &map)?.unwrap_or_default(),
            relay: bool::parse(RELAY, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::connections::supported_services::SupportedServices;

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "connection {
            p2p_protocol_version = V70015
            ibd_method = HeaderFirst
            block_height = 0
            services = [Unname]
            magic_numbers = [1, 2, 3, 4]
            nonce = 0
            user_agent = Tanto tiempo
            relay = true
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            block_height: 0,
            services: BitfieldServices {
                elements: vec![SupportedServices::Unname],
            },
            magic_numbers: [1, 2, 3, 4],
            nonce: 0,
            user_agent: "Tanto tiempo".to_string(),
            relay: true,
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "connection {
                      p2p_protocol_version = V70015
            ibd_method=                                 HeaderFirst
            block_height = 0
            services = [Unname]
            magic_numbers = [1, 2, 3, 4]
            nonce = 0
            user_agent = Tanto tiempo
            relay = true
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            block_height: 0,
            services: BitfieldServices {
                elements: vec![SupportedServices::Unname],
            },
            magic_numbers: [1, 2, 3, 4],
            nonce: 0,
            user_agent: "Tanto tiempo".to_string(),
            relay: true,
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let configuration = "connection {
            p2p_protocol_version = V70015
            block_height = 0
            services = [Unname]
            magic_numbers = [1, 2, 3, 4]
            nonce = 0
            user_agent = Tanto tiempo
            relay = true
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), connection_result);
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let configuration = "connection {
            p2p_protocol_version = V70015
            ibd_method = HeaderFirst
            ibd_method = HeaderFirst
            block_height = 0
            services = [Unname]
            magic_numbers = [1, 2, 3, 4]
            nonce = 0
            user_agent = Tanto tiempo
            relay = true
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            block_height: 0,
            services: BitfieldServices {
                elements: vec![SupportedServices::Unname],
            },
            magic_numbers: [1, 2, 3, 4],
            nonce: 0,
            user_agent: "Tanto tiempo".to_string(),
            relay: true,
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test05_does_not_accept_input_with_not_information() {
        let configuration = "";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        assert_eq!(Err(ErrorConfiguration::ValueNotFound), connection_result);
    }
}
