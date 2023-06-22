use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
};

use crate::block_structure::hash::hash256d_reduce;

use super::{
    addr_message::AddrMessage,
    alert_message::AlertMessage,
    block_message::BlockMessage,
    command_name::CommandName,
    error_message::ErrorMessage,
    fee_filter_message::FeeFilterMessage,
    get_data_message::GetDataMessage,
    get_headers_message::GetHeadersMessage,
    headers_message::HeadersMessage,
    inventory_message::InventoryMessage,
    message_header::{MagicType, MessageHeader},
    ping_message::PingMessage,
    pong_message::PongMessage,
    send_cmpct_message::SendCmpctMessage,
    send_headers_message::SendHeadersMessage,
    tx_message::TxMessage,
    verack_message::VerackMessage,
    version_message::VersionMessage,
};

use std::io::{Read, Write};

pub const CHECKSUM_EMPTY_PAYLOAD: MagicType = [0x5d, 0xf6, 0xe0, 0xe2];

pub trait Message: SerializableInternalOrder + DeserializableInternalOrder {
    fn serialize_message(
        stream: &mut dyn Write,
        magic_numbers: MagicType,
        payload: &dyn SerializableInternalOrder,
    ) -> Result<(), ErrorSerialization> {
        let mut serialized_payload: Vec<u8> = Vec::new();
        payload.io_serialize(&mut serialized_payload)?;
        let serialized_payload: &[u8] = &serialized_payload;

        let header = MessageHeader {
            magic_numbers,
            command_name: Self::get_command_name(),
            payload_size: serialized_payload.len() as u32,
            checksum: hash256d_reduce(serialized_payload)?,
        };

        header.io_serialize(stream)?;
        serialized_payload.io_serialize(stream)?;

        Ok(())
    }

    ///
    /// ### Error
    ///  * `ErrorSerialization::ErrorSerialization`: It will appear when there is an error in the serialization
    ///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorSerialization::ErrorWhileReading`: It will appear when there is an error in the reading from a stream
    fn deserialize_message(
        stream: &mut dyn Read,
        message_header: MessageHeader,
    ) -> Result<Self, ErrorSerialization> {
        let mut buffer: Vec<u8> = vec![0; message_header.payload_size as usize];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorSerialization::ErrorWhileReading);
        }
        let mut buffer: &[u8] = &buffer[..];
        let message = Self::io_deserialize(&mut buffer)?;

        let mut serialized_message: Vec<u8> = Vec::new();
        message.io_serialize(&mut serialized_message)?;

        let length = serialized_message.len();
        if length != message_header.payload_size as usize {
            return Err(ErrorSerialization::ErrorInDeserialization(format!(
                "Payload size {:?} in {:?} isn't the same as receive: {:?}",
                length,
                Self::get_command_name(),
                message_header.payload_size
            )));
        }

        let checksum = Self::calculate_checksum(&serialized_message)?;
        if !checksum.eq(&message_header.checksum) {
            return Err(ErrorSerialization::ErrorInDeserialization(format!(
                "Checksum {:?} in {:?}  isn't the same as receive: {:?}",
                checksum,
                Self::get_command_name(),
                message_header.checksum
            )));
        }

        Ok(message)
    }

    fn calculate_checksum(serialized_message: &[u8]) -> Result<[u8; 4], ErrorSerialization> {
        hash256d_reduce(serialized_message)
    }

    fn get_command_name() -> CommandName;
}

///  * `ErrorSerialization::ErrorSerialization`: It will appear when there is an error in the serialization
///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when there is an error in the deserialization
///  * `ErrorSerialization::ErrorWhileReading`: It will appear when there is an error in the reading from a stream
pub fn deserialize_until_found<RW: Read + Write>(
    stream: &mut RW,
    search_name: CommandName,
) -> Result<MessageHeader, ErrorMessage> {
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
            CommandName::Version => ignore_message::<VersionMessage>(stream, header)?,
            CommandName::Verack => ignore_message::<VerackMessage>(stream, header)?,
            CommandName::GetHeaders => ignore_message::<GetHeadersMessage>(stream, header)?,
            CommandName::Headers => ignore_message::<HeadersMessage>(stream, header)?,
            CommandName::Inventory => ignore_message::<InventoryMessage>(stream, header)?,
            CommandName::Block => ignore_message::<BlockMessage>(stream, header)?,
            CommandName::Ping => {
                let ping = PingMessage::deserialize_message(stream, header)?;

                let pong = PongMessage { nonce: ping.nonce };

                PongMessage::serialize_message(stream, magic_bytes, &pong)?;
            }
            CommandName::Pong => ignore_message::<PongMessage>(stream, header)?,
            CommandName::SendHeaders => ignore_message::<SendHeadersMessage>(stream, header)?,
            CommandName::SendCmpct => ignore_message::<SendCmpctMessage>(stream, header)?,
            CommandName::Addr => ignore_message::<AddrMessage>(stream, header)?,
            CommandName::FeeFilter => ignore_message::<FeeFilterMessage>(stream, header)?,
            CommandName::GetData => ignore_message::<GetDataMessage>(stream, header)?,
            CommandName::Alert => ignore_message::<AlertMessage>(stream, header)?,
            CommandName::Tx => ignore_message::<TxMessage>(stream, header)?,
        }
    }
}

pub fn ignore_message<M: Message>(
    stream: &mut dyn Read,
    header: MessageHeader,
) -> Result<(), ErrorSerialization> {
    let _ = M::deserialize_message(stream, header)?;
    Ok(())
}
