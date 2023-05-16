use crate::serialization::{
    serializable::Serializable,
    serializable_big_endian::SerializableBigEndian,
    deserializable::Deserializable, 
    deserializable_big_endian::DeserializableBigEndian,
    error_serialization::ErrorSerialization,
};

use crate::messages::{
    bitfield_services::BitfieldServices,
};  

use std::net::{
    Ipv6Addr,
};

use std::io::{
    Read,
    Write,
};

#[derive(Debug)]
pub struct NetworkIpAddres {
    pub time: u32,
    pub services: BitfieldServices,
    pub ip_address: Ipv6Addr,
    pub port: u16,
}

impl Serializable for NetworkIpAddres {
    
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        
        self.time.serialize(stream)?;
        self.services.serialize(stream)?;
        self.ip_address.serialize_big_endian(stream)?;
        self.port.serialize_big_endian(stream)?;

        Ok(())
    }
}

impl Deserializable for NetworkIpAddres {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {

        Ok(NetworkIpAddres {
            time: u32::deserialize(stream)?,
            services: BitfieldServices::deserialize(stream)?,
            ip_address: Ipv6Addr::deserialize_big_endian(stream)?,
            port: u16::deserialize_big_endian(stream)?,
        })
    }
}