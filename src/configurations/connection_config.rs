use super::deserializable::deserialize;
use super::deserializable_structure::DeserializeStructure;
use super::parse_error::ParseError;
use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};
use std::collections::HashMap;
use std::net::IpAddr;

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

impl<'d> DeserializeStructure<'d> for ConnectionConfig {
    type Value = ConnectionConfig;

    fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ParseError> {
        Ok(ConnectionConfig {
            dns_address: deserialize::<IpAddr>(DNS_ADDRESS, &settings_dictionary)?,
            p2p_protocol_version: deserialize::<ProtocolVersionP2P>(
                P2P_PROTOCOL_VERSION,
                &settings_dictionary,
            )?,
            ibd_method: deserialize::<IBDMethod>(IBD_METHOD, &settings_dictionary)?,
        })
    }
}
