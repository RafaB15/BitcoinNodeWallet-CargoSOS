use crate::connections::{
    p2p_protocol::ProtocolVersionP2P,
    ibd_methods::IBDMethod,
    suppored_services::SupportedServices
};

use std::net::Ipv6Addr;

pub struct Node {
    protocol_version: ProtocolVersionP2P,
    ibd_method: IBDMethod,
    peers_addrs: Vec<Ipv6Addr>,
    services: SupportedServices,
}


impl Node {
    
    pub fn new(
        protocol_version: ProtocolVersionP2P,
        ibd_method: IBDMethod,
        services: SupportedServices) -> Self {
        Node {
            protocol_version,
            ibd_method,
            peers_addrs: vec![],
            services,
        }
    }

}