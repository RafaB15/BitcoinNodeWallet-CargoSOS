use crate::serialization::{
    serializable_little_endian::SerializableLittleEndian,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, 
};

use std::convert::{
    From,
    Into,
};

use std::io::{
    Read,
    Write,
};

use std::cmp::{
    PartialOrd,
    Ordering,
};

const COMPACT256_BASE: u32 = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compact256 {
    pub mantissa: [u8; 3],
    pub exponent: u8,
}

impl From<u32> for Compact256 {
 
    fn from(value: u32) -> Self {

        let values: [u8; 4] = value.to_be_bytes();
        
        Compact256 {
            exponent: values[0],
            mantissa: [values[1], values[2], values[3]],
        }
    }
}

impl From<[u8; 32]> for Compact256 {

    fn from(value: [u8; 32]) -> Self {

        // Convertir los primeros 4 bytes del array a un u32 en formato little-endian
        let mantissa_u32 = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);

        // Encontrar el exponente del número
        let mut exponent = 0;
        let mut mantissa_u32_copy = mantissa_u32;
        while mantissa_u32_copy > 0 && exponent < COMPACT256_BASE {
            mantissa_u32_copy >>= 1;
            exponent += 1;
        }

        // Construir el Compact256
        let mut mantissa = [0u8; 3];
        let mantissa_u32 = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);

        for (index, mantissa_byte) in mantissa.iter_mut().enumerate() {
            *mantissa_byte = (mantissa_u32 >> (8 * index)) as u8;
        }

        Compact256 { mantissa, exponent: exponent as u8 }
    }
}

impl Into<u32> for Compact256 {

    fn into(self) -> u32 {
        u32::from_be_bytes([
            self.exponent, 
            self.mantissa[0], 
            self.mantissa[1], 
            self.mantissa[2],
        ])
    }
}

impl PartialOrd for Compact256 {
    
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.mantissa.partial_cmp(&other.mantissa) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.exponent.partial_cmp(&other.exponent)
    }
}

impl SerializableLittleEndian for Compact256 {
    
    fn le_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        
        let value: u32 = (*self).into();
        value.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableLittleEndian for Compact256 {

    fn le_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
    
        let value = u32::le_deserialize(stream)?;
        Ok(value.into())
    }
}