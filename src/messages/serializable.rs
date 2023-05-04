use std::io::Write;

pub trait Serializable {
    
    fn serialize(&self, stream: &mut dyn Write);
    
}