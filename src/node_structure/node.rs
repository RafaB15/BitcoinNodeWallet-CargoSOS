use crate::connections::{
    p2p_protocol::ProtocolVersionP2P,
    ibd_methods::IBDMethod,
    suppored_services::SupportedServices
};

use std::net::{
    Ipv6Addr,
    SocketAddr
};
use crate::messages::version_message::VersionMessage;
use chrono::offset::Utc;

const IGNORE_NONCE: u64 = 0;
const IGNORE_USER_AGENT: &str = "";
const NO_NEW_TRANSACTIONS: bool = false; 


pub struct Node {
    protocol_version: ProtocolVersionP2P,
    ibd_method: IBDMethod,
    peers_addrs: Vec<Ipv6Addr>,
    services: SupportedServices,
    blockchain_height: i32,
}


impl Node {
    
    pub fn new(
        protocol_version: ProtocolVersionP2P,
        ibd_method: IBDMethod,
        services: SupportedServices,
        blockchain_height: i32,
    ) -> Self {
        Node {
            protocol_version,
            ibd_method,
            peers_addrs: vec![],
            services,
            blockchain_height
        }
    }


    ///Function that tries to build a version message with the current information of the node
    pub fn build_version_message(
        &self,
        recv_addr: Ipv6Addr,
        recv_services: SupportedServices,
        trans_addr: Ipv6Addr,
        port: u16,
        nonce: u64,
        user_agent: String,
        relay: bool
    ) ->  VersionMessage {

        let timestamp = Utc::now();
        
        VersionMessage::new(
            self.protocol_version, 
            self.services, 
            timestamp, 
            recv_services, 
            recv_addr, 
            port, 
            trans_addr, 
            port, 
            nonce, 
            user_agent, 
            self.blockchain_height, 
            relay)
    }
    /*
    ///Función que intenta hacer el handshake
    pub fn handshake(potential_peers: Vec<SocketAddr>) {

    }
    */

}