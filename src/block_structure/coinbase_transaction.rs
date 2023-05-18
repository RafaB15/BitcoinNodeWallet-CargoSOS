use super::hash::HashType;
use crate::messages::compact_size::CompactSize;
use crate::serialization::{error_serialization::ErrorSerialization, serializable_little_endian::SerializableLittleEndian};
use std::io::Write;

pub struct CoinbaseTransaction {
    pub hash: HashType,       //should be null [32-byte null]
    pub index: u32,           //should be UINT32_MAX [0xffffffff]
    pub script_bytes: u32,    //ompactSize
    pub height: u32,          //should be script [Varies (4)]
    pub coinbase_script: u32, //should be None
    pub sequence: u32,        //should be uint32_t [4]
}

impl SerializableLittleEndian for CoinbaseTransaction {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.hash.le_serialize(stream)?;
        self.index.le_serialize(stream)?;
        CompactSize::new(self.script_bytes as u64).le_serialize(stream)?; //ver esto
        self.script_bytes.le_serialize(stream)?;
        self.height.le_serialize(stream)?;
        self.coinbase_script.le_serialize(stream)?;
        self.sequence.le_serialize(stream)?;
        Ok(())
    }
}
