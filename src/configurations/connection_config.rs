use super::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};

use std::{cmp::PartialEq, net::IpAddr};

const DNS_ADDRESS: &str = "dns_address";
const P2P_PROTOCOL_VERSION: &str = "p2p_protocol_version";
const IBD_METHOD: &str = "ibd_method";

#[derive(Debug, PartialEq)]
pub struct ConnectionConfig {
    ///Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    pub dns_address: IpAddr,
    /// Es la versión del protocolo peer to peer que se planea utilizar
    pub p2p_protocol_version: ProtocolVersionP2P,
    ///El método usado para el initial blocks download
    pub ibd_method: IBDMethod,
}

impl Parsable for ConnectionConfig {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(ConnectionConfig {
            dns_address: IpAddr::parse(DNS_ADDRESS, &map)?,
            p2p_protocol_version: ProtocolVersionP2P::parse(P2P_PROTOCOL_VERSION, &map)?,
            ibd_method: IBDMethod::parse(IBD_METHOD, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};

    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "connection = {
            dns_address = 127.0.0.1
            p2p_protocol_version = V70015
            ibd_method = HeaderFirst
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "connection = {
            dns_address =                           127.0.0.1
                      p2p_protocol_version = V70015
            ibd_method=                                 HeaderFirst
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
        };

        assert_eq!(Ok(config_connection), connection_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_values() {
        let configuration = "connection = {
            dns_address = 127.0.0.1
            p2p_protocol_version = V70015
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        assert_eq!(
            Err(ErrorConfiguration::ErrorReadableError),
            connection_result
        );
    }

    #[test]
    fn test04_accept_input_with_duplicate_value() {
        let configuration = "connection = {
            dns_address = 127.0.0.1
            p2p_protocol_version = V70015
            ibd_method = HeaderFirst
            ibd_method = HeaderFirst
        }";

        let name = "connection";
        let map = parse_structure(configuration.to_string()).unwrap();

        let connection_result = ConnectionConfig::parse(name, &map);

        let config_connection = ConnectionConfig {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
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
            Err(ErrorConfiguration::ErrorReadableError),
            connection_result
        );
    }
}
