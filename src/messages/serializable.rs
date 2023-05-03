pub trait Serializable {
    
    fn serialize(&self) -> Vec<u8>;
    
}