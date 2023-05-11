use std::net::{
    SocketAddr,
    TcpStream,
};

use crate::messages::error_message::ErrorMessage;

use bitcoin_hashes::{
    sha256d,
    Hash,
};

use crate::{connections::p2p_protocol::ProtocolVersionP2P, block_structure::block_header::BlockHeader};

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
        if let Some(last_header) = self.header_chain.last() {
            let hash_bytes: &[u8] = sha256d::Hash::hash(last_header).as_ref();
            let get_headers_message = GetHeadersMessage::new(
                self.protocol_version,
                vec![hash_bytes],
                NO_STOP_HASH,
            );
            get_headers_message.serialize(peer_stream)?;
        }
        Ok(())
    }
}