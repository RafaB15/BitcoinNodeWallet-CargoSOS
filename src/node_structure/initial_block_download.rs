use std::net::{
    SocketAddr,
    TcpStream,
};

use bitcoin_hashes::{
    sha256d,
    Hash,
};

use crate::messages::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
    get_headers_message::GetHeadersMessage,
};

use crate::{
    connections::p2p_protocol::ProtocolVersionP2P, 
    block_structure::block_header::BlockHeader
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
        
        let hash_bytes = sha256d::Hash::hash(&serialized_header);
        let hash_bytes: &[u8] = hash_bytes.as_ref();
        let hashed_header: [u8; 32] = match hash_bytes.try_into() {
            Ok(hashed_header) => hashed_header,
            Err(_) => return Err(ErrorMessage::ErrorInSerialization("While serializing last header".to_string())),
        };
        
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