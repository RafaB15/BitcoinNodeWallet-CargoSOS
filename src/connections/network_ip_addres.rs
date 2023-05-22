use crate::serialization::{
    deserializable_big_endian::DeserializableBigEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_big_endian::SerializableBigEndian,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::messages::bitfield_services::BitfieldServices;

use std::net::Ipv6Addr;

use std::io::{Read, Write};

#[derive(Debug)]
pub struct NetworkIpAddres {
    pub time: u32,
    pub services: BitfieldServices,
    pub ip_address: Ipv6Addr,
    pub port: u16,
}

impl SerializableLittleEndian for NetworkIpAddres {
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.time.le_serialize(stream)?;
        self.services.le_serialize(stream)?;
        self.ip_address.be_serialize(stream)?;
        self.port.be_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableLittleEndian for NetworkIpAddres {
    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(NetworkIpAddres {
            time: u32::le_deserialize(stream)?,
            services: BitfieldServices::le_deserialize(stream)?,
            ip_address: Ipv6Addr::be_deserialize(stream)?,
            port: u16::be_deserialize(stream)?,
        })
    }
}
