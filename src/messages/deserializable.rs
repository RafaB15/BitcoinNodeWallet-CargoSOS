use std::io::Read;

pub trait Deserializable {

    fn deserialize(stream: &mut dyn Read) -> Self;

}