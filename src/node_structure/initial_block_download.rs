use std::net::{
    SocketAddr,
    TcpStream,
};

use crate::messages::{
    error_message::ErrorMessage,
    get_headers_message::GetHeadersMessage,
};

use crate::serialization::serializable::Serializable;

use crate::{
    connections::p2p_protocol::ProtocolVersionP2P, 
    block_structure::{
        block_header::BlockHeader,
        hash::{
            HashType,
            hash256d,
        },
    },
};

const TESTNET_MAGIC_NUMBERS: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];
const NO_STOP_HASH: [u8; 32] = [0; 32];

pub struct InitialBlockDownload {
    pub protocol_version: ProtocolVersionP2P,
    pub peers_adrrs: Vec<SocketAddr>,
    pub header_chain: Vec<BlockHeader>,
}

impl InitialBlockDownload {
    pub fn new(protocol_version: ProtocolVersionP2P, peers_adrrs: Vec<SocketAddr>, header_chain: Vec<BlockHeader>) -> Self {
        InitialBlockDownload {
            protocol_version,
            peers_adrrs,
            header_chain,
        }
    }

    pub fn send_get_headers_message(&self, peer_stream: &mut TcpStream) -> Result<(), ErrorMessage>{
        let last_header = match self.header_chain.last() {
            Some(last_header) => last_header,
            None => return Err(ErrorMessage::ErrorInSerialization("While serializing last header".to_string())),
        };
        let mut serialized_header = Vec::new();

        last_header.serialize(&mut serialized_header)?;
        
        let hashed_header: HashType = hash256d(&serialized_header)?;
        
        let get_headers_message = GetHeadersMessage::new(
            TESTNET_MAGIC_NUMBERS,
            self.protocol_version.clone(),
            vec![hashed_header],
            NO_STOP_HASH,
        );
        get_headers_message.serialize(peer_stream)?;
        Ok(())
    }
}