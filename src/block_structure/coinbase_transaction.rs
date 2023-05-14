use super::hash::HashType;
use crate::serialization::{
    serializable::Serializable,
    error_serialization::ErrorSerialization,
};
use crate::messages::compact_size::CompactSize;
use std::io::Write;

pub struct CoinbaseTransaction {
    pub hash: HashType, //should be null [32-byte null]
    pub index: u32, //should be UINT32_MAX [0xffffffff]
    pub script_bytes: u32, //ompactSize
    pub height: u32, //should be script [Varies (4)]
    pub coinbase_script: u32, //should be None
    pub sequence: u32, //should be uint32_t [4]
}

impl Serializable for CoinbaseTransaction {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.hash.serialize(stream)?;
        self.index.serialize(stream)?;
        CompactSize::new(self.script_bytes as u64).serialize(stream)?; //ver esto
        self.script_bytes.serialize(stream)?;
        self.height.serialize(stream)?;
        self.coinbase_script.serialize(stream)?;
        self.sequence.serialize(stream)?;
        Ok(())
    }
}