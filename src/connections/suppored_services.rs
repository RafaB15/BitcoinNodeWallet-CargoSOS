use super::error_connection::ErrorConnection;

use crate::messages::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

#[derive(Debug, std::cmp::PartialEq, Copy, Clone)]
///
pub enum SupportedServices {
    Unname,
    NodeNetwork,
    NodeGetUTXO,
    NodeBloom,
    NodeWitness,
    NodeXThin,
    NodeNetworkLimited,
}

///Implementación del trait que permite hacer parse
impl std::str::FromStr for SupportedServices {
    type Err = ErrorConnection;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err(ErrorConnection::ErrorInvalidInputParse)
    }

}

impl std::convert::TryFrom<u64> for SupportedServices {
    type Error = ErrorConnection;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SupportedServices::Unname),
            1 => Ok(SupportedServices::NodeNetwork),
            2 => Ok(SupportedServices::NodeGetUTXO),
            4 => Ok(SupportedServices::NodeBloom),
            8 => Ok(SupportedServices::NodeWitness),
            10 => Ok(SupportedServices::NodeXThin),
            400 => Ok(SupportedServices::NodeNetworkLimited),
            _ => return Err(ErrorConnection::ErrorInvalidInputParse),
        }
    }
}

impl std::convert::TryInto<u64> for SupportedServices {
    type Error = ErrorConnection;

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            SupportedServices::Unname => Ok(0),
            SupportedServices::NodeNetwork => Ok(1),
            SupportedServices::NodeGetUTXO => Ok(2),
            SupportedServices::NodeBloom => Ok(4),
            SupportedServices::NodeWitness => Ok(8),
            SupportedServices::NodeXThin => Ok(10),
            SupportedServices::NodeNetworkLimited => Ok(400),
        }
    }
}

impl Serializable for SupportedServices {
    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorMessage> {
        let version: u64 = match (*self).try_into() {
            Ok(version) => version,
            _ => return Err(ErrorMessage::ErrorWhileWriting),
        };

        match stream.write(&version.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}

impl Deserializable for SupportedServices {

    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorMessage> {
        let supported_servicies = <u64 as Deserializable>::deserialize(stream)?;
        match supported_servicies.try_into() {
            Ok(supported_servicies) => Ok(supported_servicies),
            _ => Err(ErrorMessage::ErrorInDeserialization),
        }
    }
}