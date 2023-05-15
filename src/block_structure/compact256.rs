
#[derive(PartialEq)]
pub struct Compact256 {
    pub mantissa: [u8; 3],
    pub exponent: u8,
    pub base: u8,
    pub bytes_in_significand: u8,
}
 
impl Compact256 {
    pub fn to_u32(&self) -> u32 {
        // Convert the mantissa bytes to a u32 value
        let mut mantissa_u32 = 0;
        for byte in self.mantissa.iter().take(self.bytes_in_significand as usize) {
            mantissa_u32 <<= 8;
            mantissa_u32 |= *byte as u32;
        }
        
        // Apply the exponent to the mantissa
        let adjusted_mantissa = mantissa_u32 * (self.base as u32).pow(self.exponent as u32);
        
        // Truncate the adjusted mantissa to a u32 value
        adjusted_mantissa as u32
    }

    pub fn from_u32(value: u32, base: u8, bytes_in_significand: u8) -> Compact256 {
        let mut mantissa: [u8; 3] = [0; 3];
        let mut mantissa_u32 = value as u64;
        let mut exponent: u8 = 0;
        
        let mut index = 0;
        while mantissa_u32 > 0 && exponent < 255 {
        if let Some(byte_ref) = mantissa.get_mut(index) {
            *byte_ref = (mantissa_u32 & 0xFF) as u8;
        }
        mantissa_u32 >>= 8;
        exponent += 1;
        index = (exponent as usize) / 8;
}
        
        Compact256 {
            mantissa,
            exponent,
            base,
            bytes_in_significand,
        }
    }
} 
