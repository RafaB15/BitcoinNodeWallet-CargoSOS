use super::hash::{HashType, HASH_TYPE_SIZE};

use crate::serialization::{
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization, serializable_little_endian::SerializableLittleEndian,
};

use std::{
    cmp::{Ordering, PartialOrd},
    convert::{From, Into, TryFrom},
    io::{Read, Write},
};

const BYTES_OF_SIGNIFICAND: u8 = 3;
const MAX_EXPONENT: u8 = 0x1F;

/// It represents a number of 256 bits with 4 bytes
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

impl From<Compact256> for u32 {
    fn from(value: Compact256) -> Self {
        u32::from_be_bytes([
            value.exponent,
            value.mantissa[0],
            value.mantissa[1],
            value.mantissa[2],
        ])
    }
}

impl TryFrom<HashType> for Compact256 {
    type Error = ErrorSerialization;

    fn try_from(value: HashType) -> Result<Self, Self::Error> {
        let mut exponent: u8 = MAX_EXPONENT;
        let mut position: usize = 0;
        for i in 0..HASH_TYPE_SIZE {
            match value.get(i) {
                Some(0) => exponent -= 1,
                Some(_) => {
                    position = i;
                    break;
                }
                None => {
                    return Err(ErrorSerialization::ErrorInSerialization(format!(
                        "Error while reading the hash256d in the position {:?}",
                        value,
                    )))?
                }
            }
        }

        let mut mantissa: [u8; BYTES_OF_SIGNIFICAND as usize] = [0; BYTES_OF_SIGNIFICAND as usize];

        for i in 0..BYTES_OF_SIGNIFICAND {
            match value.get(position + (i as usize)) {
                Some(value) => {
                    mantissa[i as usize] = *value;
                }
                None => break,
            }
        }

        Ok(Compact256 { mantissa, exponent })
    }
}

impl PartialOrd for Compact256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.exponent.partial_cmp(&other.exponent) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.mantissa.partial_cmp(&other.mantissa)
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
