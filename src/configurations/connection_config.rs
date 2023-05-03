use super::deserializable::deserializar;
use super::estructura_deserializable::EstructuraDeserializable;
use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};
use super::parse_error::ErroresParseo;
use std::collections::HashMap;
use std::net::IpAddr;

const CONNECTION_CONFIG: &str = "Connection";

const DNS_ADDRESS: &str = "dns_address";
const P2P_PROTOCOL_VERSION: &str = "p2p_protocol_version";
const IBD_METHOD: &str = "ibd_method";

#[derive(Debug, std::cmp::PartialEq)]
pub struct ConnectionConfig {
    ///Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    pub dns_address: IpAddr,
    /// Es la versión del protocolo peer to peer que se planea utilizar
    pub p2p_protocol_version: ProtocolVersionP2P,
    ///El método usado para el initial blocks download
    pub ibd_method: IBDMethod,
}

impl<'d> EstructuraDeserializable<'d> for ConnectionConfig {
    type Valor = ConnectionConfig;

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ErroresParseo> {
        Ok(ConnectionConfig { 
            dns_address: deserializar::<IpAddr>(DNS_ADDRESS, &settings_dictionary)?,
            p2p_protocol_version: deserializar::<ProtocolVersionP2P>(P2P_PROTOCOL_VERSION, &settings_dictionary)?,
            ibd_method: deserializar::<IBDMethod>(IBD_METHOD, &settings_dictionary)?,
        })
    }

    fn nombre() -> String {
        CONNECTION_CONFIG.to_string()
    }
}