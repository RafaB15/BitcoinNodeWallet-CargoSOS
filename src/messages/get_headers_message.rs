use crate::connections::p2p_protocol::ProtocolVersionP2P;

use super::compact_size::CompactSize;

use crate::serialization::{
    serializable::Serializable,
    error_serialization::ErrorSerialization,
};

use std::io::Write;

use crate::block_structure::hash::{
    hash256d_reduce,
    HashType,
};

pub const GET_HEADERS_TYPE: &[u8; 12] = b"getheaders\0\0";

pub struct GetHeadersMessage {
    pub magic_numbers: [u8; 4],
    pub version: ProtocolVersionP2P,
    pub header_locator_hashes: Vec<HashType>, //Lista de hashes de los headers que el recv node va a chequear si tiene
    pub stop_hash: HashType, //El hash hasta el que quiero avanzar. Todos ceros significa que quiero ir hasta el final
}

impl GetHeadersMessage {
    pub fn new(
        magic_bytes: [u8; 4],
        version: ProtocolVersionP2P,
        header_locator_hashes: Vec<HashType>,
        stop_hash: HashType,
    ) -> Self {
        GetHeadersMessage {
            magic_numbers: magic_bytes,
            version,
            header_locator_hashes,
            stop_hash,
        }
    }

    pub fn serialize_payload(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.serialize(stream)?;
        CompactSize::new(self.header_locator_hashes.len() as u64).serialize(stream)?;
        
        for hash in self.header_locator_hashes.iter() {
            hash.serialize(stream)?;
        }
        
        self.stop_hash.serialize(stream)?;
        Ok(())
    }

}

impl Serializable for GetHeadersMessage {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
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
        hash256d_reduce(&serialized_payload)?.serialize(&mut serialized_message)?;

        // payload
        serialized_payload.serialize(&mut serialized_message)?;

        serialized_message.serialize(stream)
    }
}

#[cfg(test)]
mod tests {

    use super::{
        GetHeadersMessage,
        ProtocolVersionP2P,
        CompactSize,
        
        Serializable,
        ErrorSerialization, 
        
        HashType,
        hash256d_reduce,

        GET_HEADERS_TYPE,
    };

    #[test]
    fn test01_serialize() -> Result<(), ErrorSerialization> {
        let magic_bytes: [u8; 4] = [0x55, 0x66, 0xee, 0xee];
        let version = ProtocolVersionP2P::V70015;
        let header_locator_hash: Vec<HashType> = vec![[1; 32], [2; 32], [0; 32]];
        let length = CompactSize::new(header_locator_hash.len() as u64);
        let stop_hash: HashType = [1; 32];

        let mut expected_stream: Vec<u8> = Vec::new();
        magic_bytes.serialize(&mut expected_stream)?;
        GET_HEADERS_TYPE.serialize(&mut expected_stream)?;

        let mut payload: Vec<u8> = Vec::new();
        version.serialize(&mut payload)?;
        length.serialize(&mut payload)?;
        for header_hash in header_locator_hash.iter() {
            let _ = header_hash.serialize(&mut payload)?;
        }
        stop_hash.serialize(&mut payload)?;

        (payload.len() as u32).serialize(&mut expected_stream)?;
        hash256d_reduce(&payload)?.serialize(&mut expected_stream)?;
        payload.serialize(&mut expected_stream)?;

        let get_headers_message = GetHeadersMessage::new(
            magic_bytes,
            version,
            header_locator_hash,
            stop_hash,
        );

        let mut stream: Vec<u8> = Vec::new();
        get_headers_message.serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

}

