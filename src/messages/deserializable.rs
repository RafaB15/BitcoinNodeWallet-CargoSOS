pub trait Deserializable {


    fn deserialize(data: Vec<u8>) -> Self;

}