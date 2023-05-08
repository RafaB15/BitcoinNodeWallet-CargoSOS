use super::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

pub struct GetHeadersMessage {
    pub version: u32,
    pub hash_count: u8,
    pub block_locator_hashes: Vec<[u8; 32]>,
    pub hash_stop: [u8; 32],
}