use super::compact_size::CompactSize;
use crate::block_structure::block_header::BlockHeader;

pub struct HeadersMessage {
    pub count: CompactSize,
    pub headers: Vec<BlockHeader>,
}