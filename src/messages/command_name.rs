use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
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
const INVENTORY_NAME: CommandNameType = [b'i', b'n', b'v', b'e', b'n', b't', b'o', b'r', b'y', b'\0', b'\0', b'\0'];
const BLOCK_NAME: CommandNameType = [b'b', b'l', b'o', b'c', b'k', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];

#[derive(Debug, Copy, Clone)]
pub enum CommandName {
    Version,
    Verack,
    GetHeaders,
    Headers,
    Inventory,
    Block, 
}

impl TryInto<CommandNameType> for CommandName {
    type Error = ErrorSerialization;

    fn try_into(self) -> Result<CommandNameType, Self::Error> {
        let command_name = match self {
            CommandName::Version => VERSION_NAME,
            CommandName::Verack => VERACK_NAME,
            CommandName::GetHeaders => GET_HEADERS_NAME,
            CommandName::Headers => HEADERS_NAME,
            CommandName::Inventory => INVENTORY_NAME,
            CommandName::Block => BLOCK_NAME,
        };

        Ok(command_name)
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
            _ => Err(ErrorSerialization::ErrorInDeserialization("Invalid command name".to_string())),
        }
    }
}

impl Serializable for CommandName {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {

        let command_name: CommandNameType = (*self).try_into()?;
        command_name.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for CommandName {
    
    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {

        let command_name: CommandNameType = CommandNameType::deserialize(stream)?;
        let command_name: CommandName = command_name.try_into()?;

        Ok(command_name)
    }
}