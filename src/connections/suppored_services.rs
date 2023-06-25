use super::error_connection::ErrorConnection;

use crate::serialization::{
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_little_endian::SerializableLittleEndian,
};

use crate::configurations::{
    error_configuration::ErrorConfiguration,
    parsable::{value_from_map, KeyValueMap, Parsable},
};

use std::{cmp::PartialEq, convert::TryFrom, convert::TryInto, str::FromStr};

const NODE_UNNAME: u64 = 0x00;
const NODE_NETWORK: u64 = 0x01;
const NODE_GET_UTXO: u64 = 0x02;
const NODE_BLOOM: u64 = 0x04;
const NODE_WITNESS: u64 = 0x08;
const NODE_XTHIN: u64 = 0x10;
const NODE_NETWORK_LIMITED: u64 = 0x0400;

/// It's the representation of the supported services of a node
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SupportedServices {
    Unname,
    NodeNetwork,
    NodeGetUTXO,
    NodeBloom,
    NodeWitness,
    NodeXThin,
    NodeNetworkLimited,
}

impl FromStr for SupportedServices {
    type Err = ErrorConfiguration;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Unname" => Ok(SupportedServices::Unname),
            "NodeNetwork" => Ok(SupportedServices::NodeNetwork),
            "NodeGetUTXO" => Ok(SupportedServices::NodeGetUTXO),
            "NodeBloom" => Ok(SupportedServices::NodeBloom),
            "NodeWitness" => Ok(SupportedServices::NodeWitness),
            "NodeXThin" => Ok(SupportedServices::NodeXThin),
            "NodeNetworkLimited" => Ok(SupportedServices::NodeNetworkLimited),
            _ => Err(ErrorConfiguration::ErrorCantParseValue(format!(
                "Supported services of {:?}",
                value
            ))),
        }
    }
}

impl Parsable for SupportedServices {
    fn parse(name: &str, map: &KeyValueMap) -> Result<Self, ErrorConfiguration> {
        let value = value_from_map(name.to_string(), map)?;
        value.parse::<SupportedServices>()
    }
}

impl TryFrom<u64> for SupportedServices {
    type Error = ErrorConfiguration;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            NODE_UNNAME => Ok(SupportedServices::Unname),
            NODE_NETWORK => Ok(SupportedServices::NodeNetwork),
            NODE_GET_UTXO => Ok(SupportedServices::NodeGetUTXO),
            NODE_BLOOM => Ok(SupportedServices::NodeBloom),
            NODE_WITNESS => Ok(SupportedServices::NodeWitness),
            NODE_XTHIN => Ok(SupportedServices::NodeXThin),
            NODE_NETWORK_LIMITED => Ok(SupportedServices::NodeNetworkLimited),
            _ => Err(ErrorConfiguration::ErrorCantParseValue(format!(
                "Supported services of {:?}",
                value
            ))),
        }
    }
}

impl TryInto<u64> for SupportedServices {
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

impl SerializableLittleEndian for SupportedServices {
    fn le_serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        let version: u64 = match (*self).try_into() {
            Ok(version) => version,
            _ => {
                return Err(ErrorSerialization::ErrorInSerialization(format!(
                    "While serializing supported services {:?}",
                    self
                )))
            }
        };

        version.le_serialize(stream)
    }
}

impl DeserializableLittleEndian for SupportedServices {
    fn le_deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let supported_servicies = u64::le_deserialize(stream)?;
        match supported_servicies.try_into() {
            Ok(supported_servicies) => Ok(supported_servicies),
            _ => Err(ErrorSerialization::ErrorInDeserialization(format!(
                "While deserializing supported services {:?}",
                supported_servicies
            ))),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{
        DeserializableLittleEndian, ErrorSerialization, SerializableLittleEndian, SupportedServices,
    };

    #[test]
    fn test01_serialize_correctly_supported_services() -> Result<(), ErrorSerialization> {
        let expected_stream: Vec<u8> = vec![0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let mut stream: Vec<u8> = Vec::new();
        let services = SupportedServices::NodeNetworkLimited;

        services.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize_correctly_supported_services() -> Result<(), ErrorSerialization> {
        let stream: Vec<u8> = vec![0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &stream;

        let expected_services = SupportedServices::NodeNetworkLimited;

        let services = SupportedServices::le_deserialize(&mut stream)?;

        assert_eq!(expected_services, services);

        Ok(())
    }
}
