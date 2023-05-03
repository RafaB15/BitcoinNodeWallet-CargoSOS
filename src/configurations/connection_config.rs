use super::deserializable::Deserializable;
use super::estructura_deserializable::EstructuraDeserializable;
use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};
use super::parse_error::ErroresParseo;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, std::cmp::PartialEq)]
pub struct ConnectionConfig {
    ///Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    pub dns_address: IpAddr,
    /// Es la versión del protocolo peer to peer que se planea utilizar
    pub p2p_protocol_version: ProtocolVersionP2P,
    ///El método usado para el initial blocks download
    pub ibd_method: IBDMethod,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
        }
    }
}

impl<'d> EstructuraDeserializable<'d> for ConnectionConfig {
    type Valor = ConnectionConfig;

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ErroresParseo> {
        let connection_config: ConnectionConfig = ConnectionConfig::default();

        connection_config
            .p2p_protocol_version
            .deserializar(&settings_dictionary)?;
        connection_config
            .ibd_method
            .deserializar(&settings_dictionary)?;
        connection_config
            .dns_address
            .deserializar(&settings_dictionary)?;

        Ok(connection_config)
    }

    fn nombre() -> String {
        "Connection".to_string()
    }
}