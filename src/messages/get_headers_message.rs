use crate::connections::p2p_protocol::ProtocolVersionP2P;

use super::{
    compact_size::CompactSize,
    message_header::MessageHeader,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Read,
    Write,
};

use crate::block_structure::hash::{
    HashType,
    hash256d_reduce,
};

pub struct GetHeadersMessage {
    pub version: ProtocolVersionP2P,
    pub header_locator_hashes: Vec<HashType>, //Lista de hashes de los headers que el recv node va a chequear si tiene
    pub stop_hash: HashType, //El hash hasta el que quiero avanzar. Todos ceros significa que quiero ir hasta el final
}

impl GetHeadersMessage {
    pub fn new(
        version: ProtocolVersionP2P,
        header_locator_hashes: Vec<HashType>,
        stop_hash: HashType,
    ) -> Self {
        GetHeadersMessage {
            version,
            header_locator_hashes,
            stop_hash,
        }
    }

    pub fn deserialize_message(
        stream: &mut dyn Read, 
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> 
    {
        let mut buffer: Vec<u8> = vec![0; message_header.payload_size as usize];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorWhileReading);
        }
        let mut buffer: &[u8] = &buffer[..];

        let message = Self::deserialize(&mut buffer)?;

        let mut serialized_message: Vec<u8> = Vec::new();
        message.serialize(&mut serialized_message)?;
        
        let checksum = hash256d_reduce(&serialized_message)?;
        if !checksum.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(
                format!("Checksum in get headers isn't the same: {:?} != {:?}", checksum, message_header.checksum)
            ));
        }

        Ok(message)        
    }
}

impl Serializable for GetHeadersMessage {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.serialize(stream)?;
        CompactSize::new(self.header_locator_hashes.len() as u64).serialize(stream)?;

        for hash in self.header_locator_hashes.iter() {
            hash.serialize(stream)?;
        }

        self.stop_hash.serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for GetHeadersMessage {

    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let version = ProtocolVersionP2P::deserialize(stream)?;
        let size = CompactSize::deserialize(stream)?;

        let mut header_locator_hashes: Vec<HashType> = Vec::new();
        for _ in 0..size.value {
            let header_locator_hash = HashType::deserialize(stream)?;
            header_locator_hashes.push(header_locator_hash);
        }

        let stop_hash = HashType::deserialize(stream)?;

        Ok(GetHeadersMessage { 
            version, 
            header_locator_hashes,
            stop_hash
        })
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
    };

    #[test]
    fn test01_serialize() -> Result<(), ErrorSerialization> {
        let version = ProtocolVersionP2P::V70015;
        let header_locator_hash: Vec<HashType> = vec![[1; 32], [2; 32], [0; 32]];
        let length = CompactSize::new(header_locator_hash.len() as u64);
        let stop_hash: HashType = [1; 32];

        let mut expected_stream: Vec<u8> = Vec::new();

        version.serialize(&mut expected_stream)?;
        length.serialize(&mut expected_stream)?;
        for header_hash in header_locator_hash.iter() {
            let _ = header_hash.serialize(&mut expected_stream)?;
        }
        stop_hash.serialize(&mut expected_stream)?;

        let get_headers_message = GetHeadersMessage::new(
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
