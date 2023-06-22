use crate::messages::message_header::MagicType;

/// It represents the data from ourself to give in the handshake to the peers
pub struct HandshakeData {
    pub nonce: u64,
    pub user_agent: String,
    pub relay: bool,
    pub magic_number: MagicType,
}
