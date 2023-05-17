use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
};

use crate::block_structure::{
    hash::hash256d_reduce,
};

use super::{
    command_name::CommandName,
    
    version_message::VersionMessage,
    verack_message::VerackMessage,
    get_headers_message::GetHeadersMessage,
    headers_message::HeadersMessage,
    block_message::BlockMessage,
    inventory_message::InventoryMessage,
    ping_message::PingMessage,
    pong_message::PongMessage,
    send_headers::SendHeadersMessage,
    send_cmpct::SendCmpctMessage,
    addr_message::AddrMessage,
    fee_filter_message::FeeFilterMessage,

    message_header::{
        MessageHeader,
        MagicType,
    },
    error_message::ErrorMessage,
};

use std::io::{
    Read,
    Write,
};

pub const CHECKSUM_EMPTY_PAYLOAD: MagicType = [0x5d, 0xf6, 0xe0, 0xe2];

pub trait Message: SerializableLittleEndian + DeserializableLittleEndian{
    
    fn serialize_message(
        stream: &mut dyn Write, 
        magic_numbers: MagicType,
        payload: &dyn SerializableLittleEndian,
    ) -> Result<(), ErrorSerialization> 
    {
        let mut serialized_payload: Vec<u8> = Vec::new();
        payload.le_serialize(&mut serialized_payload)?;
        let serialized_payload: &[u8] = &serialized_payload;
    
        let header = MessageHeader {
            magic_numbers,
            command_name: Self::get_command_name(),
            payload_size: serialized_payload.len() as u32,
            checksum: hash256d_reduce(serialized_payload)?,
        };
    
        header.le_serialize(stream)?;
        serialized_payload.le_serialize(stream)?;        
    
        Ok(())
    }

    fn deserialize_message(
        stream: &mut dyn Read, 
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> 
    {
        let mut buffer: Vec<u8> = vec![0; message_header.payload_size as usize];
        if stream.read_exact(&mut buffer).is_err() {

            return Err(ErrorSerialization::ErrorWhileReading);
        }
        let mut buffer: &[u8] = &buffer[..];
        let message = Self::le_deserialize(&mut buffer)?;

        let mut serialized_message: Vec<u8> = Vec::new();
        message.le_serialize(&mut serialized_message)?;

        let length = serialized_message.len();
        if length != message_header.payload_size as usize {
            return Err(ErrorSerialization::ErrorInDeserialization(
                format!(
                    "Payload size {:?} in {:?} isn't the same as receive: {:?}", 
                    length, 
                    Self::get_command_name(), 
                    message_header.payload_size
                )
            ));
        }
        
        let checksum = Self::calculate_checksum(&serialized_message)?;
        if !checksum.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(
                format!(
                    "Checksum {:?} in {:?}  isn't the same as receive: {:?}", 
                    checksum, 
                    Self::get_command_name(),
                    message_header.checksum
                )
            ));
        }

        Ok(message)
    }

    fn calculate_checksum(
        serialized_message: &[u8],
    ) -> Result<[u8; 4], ErrorSerialization> {        
        hash256d_reduce(serialized_message)
    }

    fn get_command_name() -> CommandName;
}

pub fn deserialize_until_found<RW : Read + Write>(
    stream: &mut RW,
    search_name: CommandName,
) -> Result<MessageHeader, ErrorMessage> 
{
    loop {
        let header = match MessageHeader::deserialize_header(stream) {
            Ok(header) => header,
            Err(error) => return Err(error.into()),
        };

        if header.command_name == search_name {
            return Ok(header);
        }

        let magic_bytes = header.magic_numbers;

        match header.command_name {
            CommandName::Version => {
                let _ = VersionMessage::deserialize_message(stream, header)?;
            },
            CommandName::Verack => {
                let _ = VerackMessage::deserialize_message(stream, header)?;
            },
            CommandName::GetHeaders => {
                let _ = GetHeadersMessage::deserialize_message(stream, header)?;
            },
            CommandName::Headers => {
                let _ = HeadersMessage::deserialize_message(stream, header)?;
            },
            CommandName::Inventory => {
                let _ = InventoryMessage::deserialize_message(stream, header)?;
            },
            CommandName::Block => {
                let _ = BlockMessage::deserialize_message(stream, header)?;
            },
            CommandName::Ping => {
                let ping = PingMessage::deserialize_message(stream, header)?;

                let pong = PongMessage {
                    nonce: ping.nonce,
                };

                PongMessage::serialize_message(
                    stream,
                    magic_bytes,
                    &pong,
                )?;
            },
            CommandName::Pong => {
                let _ = PongMessage::deserialize_message(stream, header)?;
            },
            CommandName::SendHeaders => {
                let _ = SendHeadersMessage::deserialize_message(stream, header)?;
            },
            CommandName::SendCmpct => {
                let _ = SendCmpctMessage::deserialize_message(stream, header)?;
            },
            CommandName::Addr => {
                let _ = AddrMessage::deserialize_message(stream, header)?;
            },
            CommandName::FeeFilter => {
                let _ = FeeFilterMessage::deserialize_message(stream, header)?;
            },
        }
    }
}