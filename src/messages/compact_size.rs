use super::deserializable::Deserializable;
use super::serializable::Serializable;
use super::error_message::ErrorMessage;
use std::io::Write;

const MAX_U8:  u64 = 0xFC;
const MAX_U16: u64 = 0xFFFF;
const MAX_U32: u64 = 0xFFFFFFFF;

const PREFIX_U16: u8 = 0xFD;
const PREFIX_U32: u8 = 0xFE;
const PREFIX_U64: u8 = 0xFF;


pub struct CompactSize {
    pub value: u64,
}

impl CompactSize {
    pub fn new(value: u64) -> CompactSize {
        CompactSize {
            value
        }
    }
}

impl Serializable for CompactSize {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorMessage> {
        if self.value <= MAX_U8 {
            (self.value as u8).serialize(stream)?;

        } else if self.value <= MAX_U16 {
            PREFIX_U16.serialize(stream)?;
            (self.value as u16).serialize(stream)?;

        } else if self.value <= MAX_U32 {
            PREFIX_U32.serialize(stream)?;
            (self.value as u32).serialize(stream)?;

        } else {
            PREFIX_U64.serialize(stream)?;
            (self.value as u64).serialize(stream)?;
        }

        Ok(())
    }
}

impl Deserializable for CompactSize {

    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorMessage> {
        let mut buffer = [0u8; 1];
        if stream.read_exact(&mut buffer).is_err() {
            return Err(ErrorMessage::ErrorInDeserialization("Deserializing CompactSize".to_string()));
        }

        match buffer {
            [PREFIX_U16] => Ok(CompactSize::new(u16::deserialize(stream)? as u64)),
            [PREFIX_U32] => Ok(CompactSize::new(u32::deserialize(stream)? as u64)),
            [PREFIX_U64] => Ok(CompactSize::new(u64::deserialize(stream)?)),
            _ => Ok(CompactSize::new(u8::deserialize(stream)? as u64))
        }
    }
}