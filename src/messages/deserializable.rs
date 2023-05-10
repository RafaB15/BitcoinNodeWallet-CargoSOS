use std::io::Read;
use super::error_message::ErrorMessage;

pub trait Deserializable {
    type Value;

    fn deserialize(stream: &mut dyn Read) -> Result<Self::Value, ErrorMessage>;
}

pub fn get_slice<const N: usize>(buffer: &[u8], posicion: &mut usize) -> Result<[u8; N], ErrorMessage>{
    let inicio = *posicion;
    let slice: [u8; N] = match buffer[inicio..(N + inicio)].try_into() {
        Ok(slice) => slice,
        _ => return Err(ErrorMessage::ErrorInDeserialization),
    };

    *posicion += N;
    Ok(slice)
}