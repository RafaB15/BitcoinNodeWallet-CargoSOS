use crate::serialization::{
    serializable_internal_order::SerializableInternalOrder,
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
};


use std::io::{
    Read,
    Write,
};

use std::convert::{
    TryFrom,
    TryInto,
};

type CommandNameType = [u8; 12];

const VERSION_NAME: CommandNameType = [b'v', b'e', b'r', b's', b'i', b'o', b'n', b'\0', b'\0', b'\0', b'\0', b'\0'];
const VERACK_NAME: CommandNameType = [b'v', b'e', b'r', b'a', b'c', b'k', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];
const GET_HEADERS_NAME: CommandNameType = [b'g', b'e', b't', b'h', b'e', b'a', b'd', b'e', b'r', b's', b'\0', b'\0'];
const HEADERS_NAME: CommandNameType = [b'h', b'e', b'a', b'd', b'e', b'r', b's', b'\0', b'\0', b'\0', b'\0', b'\0'];
const INVENTORY_NAME: CommandNameType = [b'i', b'n', b'v', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];
const BLOCK_NAME: CommandNameType = [b'b', b'l', b'o', b'c', b'k', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];
const PING_NAME: CommandNameType = [b'p', b'i', b'n', b'g', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];
const PONG_NAME: CommandNameType = [b'p', b'o', b'n', b'g', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];
const SEND_HEADERS_NAME: CommandNameType = [b's', b'e', b'n', b'd', b'h', b'e', b'a', b'd', b'e', b'r', b's', b'\0'];
const SEND_CMPCT_NAME: CommandNameType = [b's', b'e', b'n', b'd', b'c', b'm', b'p', b'c', b't', b'\0', b'\0', b'\0'];
const ADDR_NAME: CommandNameType = [b'a', b'd', b'd', b'r', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];
const FEE_FILTER_NAME: CommandNameType = [b'f', b'e', b'e', b'f', b'i', b'l', b't', b'e', b'r', b'\0', b'\0', b'\0'];
const GET_DATA_NAME: CommandNameType = [b'g', b'e', b't', b'd', b'a', b't', b'a', b'\0', b'\0', b'\0', b'\0', b'\0'];
const ALERT_NAME: CommandNameType = [b'a', b'l', b'e', b'r', b't', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];

#[derive(Debug, Copy, Clone, std::cmp::PartialEq)]
pub enum CommandName {
    Version,
    Verack,
    GetHeaders,
    Headers,
    Inventory,
    Block, 
    Ping,
    Pong,
    SendHeaders,
    SendCmpct,
    Addr,
    FeeFilter,
    GetData,
    Alert,
}

impl From<CommandName> for CommandNameType {
    
    fn from(command_name: CommandName) -> CommandNameType {
        match command_name {
            CommandName::Version => VERSION_NAME,
            CommandName::Verack => VERACK_NAME,
            CommandName::GetHeaders => GET_HEADERS_NAME,
            CommandName::Headers => HEADERS_NAME,
            CommandName::Inventory => INVENTORY_NAME,
            CommandName::Block => BLOCK_NAME,
            CommandName::Ping => PING_NAME,
            CommandName::Pong => PONG_NAME,
            CommandName::SendHeaders => SEND_HEADERS_NAME,
            CommandName::SendCmpct => SEND_CMPCT_NAME,
            CommandName::Addr => ADDR_NAME,
            CommandName::FeeFilter => FEE_FILTER_NAME,
            CommandName::GetData => GET_DATA_NAME,
            CommandName::Alert => ALERT_NAME,
        }
    }
}

impl TryFrom<CommandNameType> for CommandName {
    type Error = ErrorSerialization;

    fn try_from(value: CommandNameType) -> Result<Self, Self::Error> {
        match value {
            VERSION_NAME => Ok(CommandName::Version),
            VERACK_NAME => Ok(CommandName::Verack),
            GET_HEADERS_NAME => Ok(CommandName::GetHeaders),
            HEADERS_NAME => Ok(CommandName::Headers),
            INVENTORY_NAME => Ok(CommandName::Inventory),
            BLOCK_NAME => Ok(CommandName::Block),
            PING_NAME => Ok(CommandName::Ping),
            PONG_NAME => Ok(CommandName::Pong),
            SEND_HEADERS_NAME => Ok(CommandName::SendHeaders),
            SEND_CMPCT_NAME => Ok(CommandName::SendCmpct),
            ADDR_NAME => Ok(CommandName::Addr),
            FEE_FILTER_NAME => Ok(CommandName::FeeFilter),
            GET_DATA_NAME => Ok(CommandName::GetData),
            ALERT_NAME => Ok(CommandName::Alert),
            _ => Err(ErrorSerialization::ErrorInDeserialization(
                format!("Invalid command name, we get: {:?}", value)
            )),
        }
    }
}

impl SerializableInternalOrder for CommandName {
    
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        let command_name: CommandNameType = (*self).into();
        command_name.io_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for CommandName {
    
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {

        let command_name = CommandNameType::io_deserialize(stream)?;
        let command_name: CommandName = command_name.try_into()?;

        Ok(command_name)
    }
}