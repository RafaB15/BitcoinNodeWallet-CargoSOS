use super::error_connection::ErrorConnection;
use std::net::{SocketAddr, ToSocketAddrs};

use crate::configurations::{
    error_configuration::ErrorConfiguration,
    parsable::{parse_structure, value_from_map, KeyValueMap, Parsable},
};

use std::cmp::PartialEq;

const SEED: &str = "seed";
const PORT: &str = "port";

#[derive(Debug, PartialEq, Clone)]
pub struct DNSSeeder {
    pub dns_addr: String,
    pub port_number: u16,
}

impl DNSSeeder {
    pub fn new(dns_addr: &str, port_number: u16) -> Self {
        DNSSeeder {
            dns_addr: dns_addr.to_string(),
            port_number,
        }
    }

    pub fn discover_peers(&self) -> Result<Vec<SocketAddr>, ErrorConnection> {
        let mut peer_addrs: Vec<SocketAddr> = Vec::new();

        if let Ok(iter) = (self.dns_addr.clone(), self.port_number).to_socket_addrs() {
            for peer_addr in iter {
                peer_addrs.push(peer_addr);
            }
        } else {
            return Err(ErrorConnection::ErrorInvalidIPOrPortNumber);
        }

        Ok(peer_addrs)
    }
}

impl Parsable for DNSSeeder {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let structure = value_from_map(name.to_string(), map)?;
        let map = parse_structure(structure)?;

        Ok(DNSSeeder {
            dns_addr: String::parse(SEED, &map)?,
            port_number: u16::parse(PORT, &map)?,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "dns_seeder {
            seed = seed.testnet.bitcoin.sprovoost.nl
            port = 18333
        }";

        let name = "dns_seeder";
        let map = parse_structure(configuration.to_string()).unwrap();

        let dns_result = DNSSeeder::parse(name, &map);

        let expected_dns = DNSSeeder {
            dns_addr: "seed.testnet.bitcoin.sprovoost.nl".to_string(),
            port_number: 18333,
        };

        assert_eq!(Ok(expected_dns), dns_result);
    }
}
