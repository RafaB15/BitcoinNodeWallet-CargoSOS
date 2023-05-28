use super::{
    parsable::{
        Parsable,
        KeyValueMap,
        value_from_map,
        parse_structure,
    },
    error_configuration::ErrorConfiguration,
};

use crate::connections::{
    ibd_methods::IBDMethod, 
    p2p_protocol::ProtocolVersionP2P,
};

use std::{
    net::IpAddr,
    cmp::PartialEq,
};

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
            p2p_protocol_version: ProtocolVersionP2P::parse(
                P2P_PROTOCOL_VERSION,
                &map,
            )?,
            ibd_method: IBDMethod::parse(IBD_METHOD, &map)?,
        })
    }
}