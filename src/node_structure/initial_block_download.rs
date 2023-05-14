use std::net::{
    SocketAddr,
    TcpStream,
};

use crate::messages::{
    error_message::ErrorMessage,
    get_headers_message::GetHeadersMessage,
};

use crate::block_structure::{
    block_chain::BlockChain,
    block::Block,
    block_header::BlockHeader,
    hash::{
        HashType,
        hash256d
    }
};

use crate::serialization::serializable::Serializable;

use crate::{
    connections::p2p_protocol::ProtocolVersionP2P, 
};

const TESTNET_MAGIC_NUMBERS: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];
const NO_STOP_HASH: HashType = [0; 32];

#[derive(Debug, Clone)]
pub struct InitialBlockDownload {
    pub protocol_version: ProtocolVersionP2P,
}

impl InitialBlockDownload {
    pub fn new(protocol_version: ProtocolVersionP2P) -> Self {
        InitialBlockDownload {
            protocol_version,
        }
    }

    fn send_get_headers_message(
        &self, 
        peer_stream: &mut TcpStream, 
        block_chain: &mut BlockChain
    ) -> Result<u32, ErrorMessage>
    {
        let last_header: &BlockHeader = &block_chain.last().header;
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
        Ok(0)
    }

    pub fn get_headers(
        &self, 
        peer_stream: &mut TcpStream, 
        block_chain: &mut BlockChain
    ) -> Result<u32, ErrorMessage> {
        todo!()
    }

    pub fn get_data(
        &self,
        peer_stream: &mut TcpStream,
        hashed_header: &HashType,
    ) -> Result<Block, ErrorMessage> {
        todo!()
    }
}