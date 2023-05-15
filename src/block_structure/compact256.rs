const COMPACT256_BASE: u32 = 256;
const BYTES_IN_SIGNIFICAND: u8 = 3;

#[derive(PartialEq)]
pub struct Compact256 {
    pub mantissa: [u8; 3],
    pub exponent: u32,
}

impl Compact256 {
    pub fn to_u32(&self) -> u32 {
        // Convert the mantissa bytes to a u32 value
        let mut mantissa_u32 = 0;
        for byte in self.mantissa.iter().take(BYTES_IN_SIGNIFICAND as usize) {
            mantissa_u32 <<= 8;
            mantissa_u32 |= *byte as u32;
        }

        // Apply the exponent to the mantissa
        let adjusted_mantissa = mantissa_u32 * (COMPACT256_BASE as u32).pow(self.exponent as u32);

        // Truncate the adjusted mantissa to a u32 value
        adjusted_mantissa as u32
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
        Compact256 { mantissa, exponent }
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
        let mut mantissa = [0; 3];
        for i in 0..3 {
            mantissa[i] = (mantissa_u32 >> (8 * i)) as u8;
        }

        Compact256 { mantissa, exponent }
    }
}
