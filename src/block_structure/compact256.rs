use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization, 
};

const COMPACT256_BASE: u32 = 256;
const BYTES_IN_SIGNIFICAND: u8 = 3;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Compact256 {
    pub mantissa: [u8; 3],
    pub exponent: u8,
}

impl Compact256 {
    pub fn to_u32(&self) -> u32 {

        u32::from_be_bytes([self.exponent, self.mantissa[0], self.mantissa[1], self.mantissa[2]])
    }

    pub fn from_u32(value: u32) -> Compact256 {
        let mut mantissa: [u8; 3] = [0; 3];
        let mut mantissa_u32 = value as u64;
        let mut exponent: u32 = 0;

        let mut index = 0;
        while mantissa_u32 > 0 && exponent < COMPACT256_BASE {
            if let Some(byte_ref) = mantissa.get_mut(index) {
                *byte_ref = (mantissa_u32 & 0xFF) as u8;
            }
            mantissa_u32 >>= 8;
            exponent += 1;
            index = (exponent as usize) / 8;
        }
        Compact256 { mantissa, exponent: exponent as u8 }
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Compact256 {
        // Convertir los primeros 4 bytes del array a un u32 en formato little-endian
        let mantissa_u32 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        // Encontrar el exponente del número
        let mut exponent = 0;
        let mut mantissa_u32_copy = mantissa_u32;
        while mantissa_u32_copy > 0 && exponent < COMPACT256_BASE {
            mantissa_u32_copy >>= 1;
            exponent += 1;
        }

        // Construir el Compact256
        let mut mantissa = [0u8; 3];
let mantissa_u32 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        for (index, mantissa_byte) in mantissa.iter_mut().enumerate() {
            *mantissa_byte = (mantissa_u32 >> (8 * index)) as u8;
        }

        Compact256 { mantissa, exponent: exponent as u8 }
    }
}

impl Serializable for Compact256 {
    
    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        let value = self.to_u32();
        value.serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for Compact256 {
    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let value = u32::deserialize(stream)?;
        Ok(Compact256::from_u32(value))
    }
}