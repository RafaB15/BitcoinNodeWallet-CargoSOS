use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};
use crate::errors::parse_error::ErroresParseo;
use std::collections::HashMap;
use super::deserializable::Deserializable;
use std::net::IpAddr;

const DNS_ADDRESS: &str = "dns_addres";

#[derive(Debug)]
pub struct ConnectionConfig {
    ///Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    pub dns_address: IpAddr,
    /// Es la versión del protocolo peer to peer que se planea utilizar
    pub p2p_protocol_version: ProtocolVersionP2P,
    ///El método usado para el initial blocks download
    pub ibd_method: IBDMethod,
}

impl ConnectionConfig {
    pub fn new<'d>(settings_dictionary: &'d HashMap<String, String>) -> Result<Self, ErroresParseo> {

        let mut connection_config: ConnectionConfig;

        connection_config.p2p_protocol_version.deserializar(settings_dictionary)?;
        connection_config.ibd_method.deserializar(settings_dictionary)?;
        connection_config.dns_address.deserializar(settings_dictionary)?;

        Ok(connection_config)
    }
}
