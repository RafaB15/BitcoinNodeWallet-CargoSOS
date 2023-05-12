use super::error_connection::ErrorConnection;

use crate::messages::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

const NODE_UNNAME: u64 = 0x00;
const NODE_NETWORK: u64 = 0x01;
const NODE_GET_UTXO: u64 = 0x02;
const NODE_BLOOM: u64 = 0x04;
const NODE_WITNESS: u64 = 0x08;
const NODE_XTHIN: u64 = 0x10;
const NODE_NETWORK_LIMITED: u64 = 0x0400;

///
#[derive(Debug, std::cmp::PartialEq, Copy, Clone)]
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
            NODE_UNNAME => Ok(SupportedServices::Unname),
            NODE_NETWORK => Ok(SupportedServices::NodeNetwork),
            NODE_GET_UTXO => Ok(SupportedServices::NodeGetUTXO),
            NODE_BLOOM => Ok(SupportedServices::NodeBloom),
            NODE_WITNESS => Ok(SupportedServices::NodeWitness),
            NODE_XTHIN => Ok(SupportedServices::NodeXThin),
            NODE_NETWORK_LIMITED => Ok(SupportedServices::NodeNetworkLimited),
            _ => Err(ErrorConnection::ErrorInvalidInputParse),
        }
    }
}

impl std::convert::TryInto<u64> for SupportedServices {
    type Error = ErrorConnection;

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            SupportedServices::Unname => Ok(NODE_UNNAME),
            SupportedServices::NodeNetwork => Ok(NODE_NETWORK),
            SupportedServices::NodeGetUTXO => Ok(NODE_GET_UTXO),
            SupportedServices::NodeBloom => Ok(NODE_BLOOM),
            SupportedServices::NodeWitness => Ok(NODE_WITNESS),
            SupportedServices::NodeXThin => Ok(NODE_XTHIN),
            SupportedServices::NodeNetworkLimited => Ok(NODE_NETWORK_LIMITED),
        }
    }
}

impl Serializable for SupportedServices {
    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorMessage> {
        let version: u64 = match (*self).try_into() {
            Ok(version) => version,
            _ => return Err(ErrorMessage::ErrorInSerialization(format!("While serializing supported services {:?}", self))),
        };

        match stream.write(&version.to_le_bytes()) {
            Ok(_) => Ok(()),
            _ => Err(ErrorMessage::ErrorWhileWriting),
        }
    }
}

impl Deserializable for SupportedServices {

    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorMessage> {
        let supported_servicies = u64::deserialize(stream)?;
        match supported_servicies.try_into() {
            Ok(supported_servicies) => Ok(supported_servicies),
            _ => Err(ErrorMessage::ErrorInDeserialization(format!("While deserializing supported services {:?}", supported_servicies))),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{
        SupportedServices,
        Serializable,
        Deserializable,
        ErrorMessage,
    };

    #[test]
    fn test01_serialize_correctly_supported_services() -> Result<(), ErrorMessage> {
        
        let expected_stream: Vec<u8> = vec![0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        
        let mut stream: Vec<u8> = Vec::new();
        let services = SupportedServices::NodeNetworkLimited;

        services.serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize_correctly_supported_services() -> Result<(), ErrorMessage> {
        
        let stream: Vec<u8> = vec![0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &stream;
        
        let expected_services = SupportedServices::NodeNetworkLimited;

        let services = SupportedServices::deserialize(&mut stream)?;

        assert_eq!(expected_services, services);

        Ok(())
    }
}