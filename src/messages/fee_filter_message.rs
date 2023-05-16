use super::{
    message_header::MessageHeader,
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable, 
    error_serialization::ErrorSerialization,
};

use crate::block_structure::{
    hash::hash256d_reduce,
};

use std::io::{
    Read, 
    Write
};

#[derive(Debug)]
pub struct FeeFilterMessage {
    pub feerate: u64,
}

impl FeeFilterMessage {
  
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
                format!("Checksum in feefilter isn't the same: {:?} != {:?}", checksum, message_header.checksum)
            ));
        }

        Ok(message)
    }
}

impl Serializable for FeeFilterMessage {

    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        self.feerate.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for FeeFilterMessage {
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {    

        Ok(FeeFilterMessage{
            feerate: u64::deserialize(stream)?,
        })
    }
}