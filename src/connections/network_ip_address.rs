use crate::serialization::{
    deserializable_big_endian::DeserializableBigEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_big_endian::SerializableBigEndian,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::messages::bitfield_services::BitfieldServices;

use std::{
    io::{Read, Write},
    net::Ipv6Addr,
};

/// It's the representation of a new potential peer to connect to
#[derive(Debug, PartialEq)]
pub struct NetworkIpAddress {
    time: u32,
    services: BitfieldServices,
    ip_address: Ipv6Addr,
    port: u16,
}

impl SerializableLittleEndian for NetworkIpAddress {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.time.le_serialize(stream)?;
        self.services.le_serialize(stream)?;
        self.ip_address.be_serialize(stream)?;
        self.port.be_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableLittleEndian for NetworkIpAddress {
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(NetworkIpAddress {
            time: u32::le_deserialize(stream)?,
            services: BitfieldServices::le_deserialize(stream)?,
            ip_address: Ipv6Addr::be_deserialize(stream)?,
            port: u16::be_deserialize(stream)?,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::connections::supported_services::SupportedServices;

    #[test]
    fn test01_serialize_correctly_network_ip_address() -> Result<(), ErrorSerialization> {
        let mut expected_stream: Vec<u8> = Vec::new();
        (1234 as u32).le_serialize(&mut expected_stream)?;
        BitfieldServices::new(vec![SupportedServices::Unname])
            .le_serialize(&mut expected_stream)?;
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).be_serialize(&mut expected_stream)?;
        (0 as u16).be_serialize(&mut expected_stream)?;

        let mut stream: Vec<u8> = Vec::new();
        let network_ip_address: NetworkIpAddress = NetworkIpAddress {
            time: 1234,
            services: BitfieldServices::new(vec![SupportedServices::Unname]),
            ip_address: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
            port: 0,
        };

        network_ip_address.le_serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize_correctly_network_ip_address() -> Result<(), ErrorSerialization> {
        let mut stream: Vec<u8> = Vec::new();
        let network_ip_address: NetworkIpAddress = NetworkIpAddress {
            time: 1234,
            services: BitfieldServices::new(vec![SupportedServices::Unname]),
            ip_address: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
            port: 0,
        };

        network_ip_address.le_serialize(&mut stream)?;

        let mut stream: &[u8] = &stream;

        let network_ip_address_deserialized = NetworkIpAddress::le_deserialize(&mut stream)?;

        assert_eq!(network_ip_address, network_ip_address_deserialized);

        Ok(())
    }
}
