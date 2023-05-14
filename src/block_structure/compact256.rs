

#[derive(PartialEq)]
pub struct Compact256 {
    pub mantisa: [u8; 3],
    pub exponente: u8,
    base: u8,
    pub bytes_in_significand: u8,
    
}
