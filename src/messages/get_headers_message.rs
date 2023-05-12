use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use bitcoin_hashes::sha256d::Hash;

use crate::connections::p2p_protocol::ProtocolVersionP2P;

pub const GET_HEADERS_TYPE: &[u8; 12] = b"getheaders\0\0";

pub struct GetHeadersMessage {
    pub magic_numbers: [u8; 4],
    pub version: ProtocolVersionP2P,
    pub header_locator_hashes: Vec<[u8; 32]>, //Lista de hashes de los headers que el recv node va a chequear si tiene
    pub stop_hash: [u8; 32], //El hash hasta el que quiero avanzar. Todos ceros significa que quiero ir hasta el final
}

impl GetHeadersMessage {
    pub fn new(
        magic_numbers: [u8; 4],
        version: ProtocolVersionP2P,
        header_locator_hashes: Vec<[u8; 32]>,
        stop_hash: [u8; 32],
    ) -> Self {
        GetHeadersMessage {
            magic_numbers,
            version,
            header_locator_hashes,
            stop_hash,
        }
    }
}


