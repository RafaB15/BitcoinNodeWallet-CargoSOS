use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
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

const MAGIC_BYTES_SIZE: usize = 4;
const MASSAGE_TYPE_SIZE: usize = 12;
const PAYLOAD_SIZE: usize = 4;
const CHECKSUM_SIZE: usize = 4;

const HEADER_SIZE: usize = MAGIC_BYTES_SIZE + MASSAGE_TYPE_SIZE + PAYLOAD_SIZE + CHECKSUM_SIZE;

pub fn serialize_message(
    stream: &mut dyn Write, 
    magic_numbers: MagicType,
    command_name: CommandName,
    payload: &dyn Serializable,
) -> Result<(), ErrorSerialization> 
{
    let mut serialized_payload: Vec<u8> = Vec::new();
    payload.serialize(&mut serialized_payload)?;
    let serialized_payload: &[u8] = &serialized_payload;

    let header = MessageHeader {
        magic_numbers,
        command_name,
        payload_size: serialized_payload.len() as u32,
        checksum: hash256d_reduce(serialized_payload)?,
    };

    header.serialize(stream)?;
    serialized_payload.serialize(stream)?;        

    Ok(())
}

pub fn deserialize_message(
    stream: &mut dyn Read,
) -> Result<MessageHeader, ErrorSerialization> 
{
    let mut buffer: Vec<u8> = vec![0; HEADER_SIZE];

    if stream.read_exact(&mut buffer).is_err() {
        return Err(ErrorSerialization::ErrorWhileReading);
    }

    let mut buffer: &[u8] = &buffer[..];

    MessageHeader::deserialize(&mut buffer)
}

pub fn deserialize_until_found<RW : Read + Write>(
    stream: &mut RW,
    search_name: CommandName,
) -> Result<MessageHeader, ErrorMessage> 
{
    loop {
        let header = match deserialize_message(stream) {
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

                serialize_message(
                    stream,
                    magic_bytes,
                    CommandName::Pong,
                    &pong,
                )?;
            },
            CommandName::Pong => {
                let _ = PongMessage::deserialize_message(stream, header)?;
            },
            CommandName::SendHeaders => {
                let _ = SendHeadersMessage::deserialize_message(stream, header)?;
            }
            CommandName::SendCmpct => {
                let _ = SendCmpctMessage::deserialize_message(stream, header)?;
            }
        }
    }
}