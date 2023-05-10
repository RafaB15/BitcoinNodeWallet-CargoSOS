use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use bitcoin_hashes::sha256d::Hash;

use crate::connections::p2p_protocol::ProtocolVersionP2P;

pub const GET_HEADERS_TYPE: [u8; 12] = *b"getheaders\0\0";

pub struct GetHeadersMessage {
    pub version: ProtocolVersionP2P,
    pub block_locator_hashes: Vec<Hash>, //Al serializar esto hay que primero poner el compact size u y luego los hashes
    pub hash_stop: Hash,
}