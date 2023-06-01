use crate::messages::message_header::MagicType;

pub struct HandshakeData {
    pub nonce: u64,
    pub user_agent: String,
    pub relay: bool,
    pub magic_number: MagicType,
}
