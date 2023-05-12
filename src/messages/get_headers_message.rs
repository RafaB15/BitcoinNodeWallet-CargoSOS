use super::{
    serializable::Serializable,
    error_message::ErrorMessage,
    compact_size::CompactSize,
};

use std::io::Write;

use bitcoin_hashes::{
    sha256d,
    Hash,
};

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
        magic_bytes: [u8; 4],
        version: ProtocolVersionP2P,
        header_locator_hashes: Vec<[u8; 32]>,
        stop_hash: [u8; 32],
    ) -> Self {
        GetHeadersMessage {
            magic_numbers: magic_bytes,
            version,
            header_locator_hashes,
            stop_hash,
        }
    }

    pub fn serialize_payload(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        self.version.serialize(stream)?;
        CompactSize::new(self.header_locator_hashes.len() as u64).serialize(stream)?;
        
        for hash in self.header_locator_hashes {
            hash.serialize(stream)?;
        }
        
        self.stop_hash.serialize(stream)?;
        Ok(())
    }

    pub fn calculate_checksum(payload: &Vec<u8>) -> Result<[u8; 4], ErrorMessage> {
        let hash_bytes: sha256d::Hash = sha256d::Hash::hash(payload); 
        let checksum: [u8; 4] = match hash_bytes[0..4].try_into() {
            Ok(checksum) => checksum,
            _ => return Err(ErrorMessage::ErrorChecksum),
        };
        Ok(checksum)
    }

}

impl Serializable for GetHeadersMessage {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        let mut serialized_message = Vec::new();
        let mut serialized_payload = Vec::new();

        // magic bytes
        self.magic_numbers.serialize(&mut serialized_message)?; 
        
        // command name
        GET_HEADERS_TYPE.serialize(&mut serialized_message)?;

        self.serialize_payload(&mut serialized_payload)?;

        // payload size
        (serialized_payload.len() as u32).serialize(&mut serialized_message)?;

        //checksum
        Self::calculate_checksum(&serialized_payload)?.serialize(&mut serialized_message)?;

        // payload
        serialized_payload.serialize(&mut serialized_message)?;

        serialized_message.serialize(stream)
    }
}


