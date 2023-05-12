use crate::messages::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_message::ErrorMessage,
};

use crate::connections::suppored_services::SupportedServices;

use std::{
    convert::TryInto,
    cmp::PartialEq,
};

#[derive(Debug, Clone)]
pub struct BitfieldServices {
    pub elements: Vec<SupportedServices>,
}

impl BitfieldServices {
    pub fn new(elements: Vec<SupportedServices>) -> Self {
        match elements.is_empty() {
            true => BitfieldServices { elements: vec![SupportedServices::Unname] },
            false => BitfieldServices { elements },
        }
    }
}

impl PartialEq for BitfieldServices {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl Serializable for BitfieldServices {
    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorMessage> {
        
        let mut sum: u64 = 0;
        for element in self.elements.clone() {
            let element_value: u64 = match element.try_into() {
                Ok(value) => value,
                _ => return Err(ErrorMessage::ErrorInSerialization("While deserializing bitfield".to_string())),
            };
            sum += element_value;
        }

        sum.serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for BitfieldServices{
    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorMessage> {
        
        let posibles_suppored = [
            SupportedServices::NodeNetwork,
            SupportedServices::NodeGetUTXO,
            SupportedServices::NodeBloom,
            SupportedServices::NodeWitness,
            SupportedServices::NodeXThin,
            SupportedServices::NodeNetworkLimited,
        ];
        
        let bitfield: u64 = u64::deserialize(stream)?;

        let mut elements: Vec<SupportedServices> = Vec::new();

        for posible_suppored in posibles_suppored {

            let supported_value: u64 = match posible_suppored.try_into() {
                Ok(value) => value,
                _ => return Err(ErrorMessage::ErrorInDeserialization(format!("While deserializing bitfield {:?}", posible_suppored))),
            };

            if bitfield & supported_value == supported_value {
                elements.push(posible_suppored);
            }
        }

        Ok(BitfieldServices::new(elements))
    }
}