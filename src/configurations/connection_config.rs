use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};
use crate::errors::parse_error::ErroresParseo;

use std::collections::HashMap;
use std::net::IpAddr;

const DNS_ADDRESS: &str = "dns_addres";
const P2P_PROTOCOL: &str = "p2p_protocol";
const IBD_METHOD: &str = "ibd_method";

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
    pub fn new(settings_dictionary: HashMap<String, String>) -> Result<Self, ErroresParseo> {
        let mut dns_address: Option<IpAddr> = None;
        let mut p2p_protocol_version: Option<ProtocolVersionP2P> = None;
        let mut ibd_method: Option<IBDMethod> = None;

        for (key, value) in settings_dictionary {
            match key.as_str() {
                DNS_ADDRESS => dns_address = Some(value.parse::<IpAddr>()?),
                P2P_PROTOCOL => p2p_protocol_version = Some(value.parse::<ProtocolVersionP2P>()?),
                IBD_METHOD => ibd_method = Some(value.parse::<IBDMethod>()?),
                _ => {
                    return Err(ErroresParseo::ParseoValorNoReconocido);
                }
            }
        }

        Self::new_from_option((dns_address, p2p_protocol_version, ibd_method))
    }

    fn new_from_option(
        values: (
            Option<IpAddr>,
            Option<ProtocolVersionP2P>,
            Option<IBDMethod>,
        ),
    ) -> Result<Self, ErroresParseo> {
        if let (Some(dns_address), Some(p2p_protocol_version), Some(ibd_method)) = values {
            Ok(ConnectionConfig {
                dns_address,
                p2p_protocol_version,
                ibd_method,
            })
        } else {
            Err(ErroresParseo::NoSuficientesValores)
        }
    }
}
