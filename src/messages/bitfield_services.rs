use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
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
        if self.elements.len() != other.elements.len() {
            return false;
        }

        let mut are_equal = true;
        self.elements.iter().for_each(|element| {
            are_equal &= other.elements.contains(element);
        });

        are_equal
    }
}

impl Serializable for BitfieldServices {
    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        
        let mut sum: u64 = 0;
        for element in self.elements.clone() {
            let element_value: u64 = match element.try_into() {
                Ok(value) => value,
                _ => return Err(ErrorSerialization::ErrorInSerialization("While deserializing bitfield".to_string())),
            };
            sum += element_value;
        }

        sum.serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for BitfieldServices{
    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        
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
                _ => return Err(ErrorSerialization::ErrorInDeserialization(format!("While deserializing bitfield {:?}", posible_suppored))),
            };

            if bitfield & supported_value == supported_value {
                elements.push(posible_suppored);
            }
        }

        Ok(BitfieldServices::new(elements))
    }
}

#[cfg(test)]
mod tests {

    use super::{
        BitfieldServices,
        SupportedServices,
        Serializable,
        Deserializable,
        ErrorSerialization,
    };

    #[test]
    fn test01_serialize_correctly_bitfield_services() -> Result<(), ErrorSerialization> {

        let expected_stream: Vec<u8> = vec![0x09, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        
        let mut stream: Vec<u8> = Vec::new();
        let services = BitfieldServices::new(vec![
            SupportedServices::NodeNetworkLimited, 
            SupportedServices::NodeWitness, 
            SupportedServices::NodeNetwork
        ]);

        services.serialize(&mut stream)?;

        assert_eq!(expected_stream, stream);

        Ok(())
    }

    #[test]
    fn test02_deserialize_correctly_bitfield_services() -> Result<(), ErrorSerialization> {

        let stream: Vec<u8> = vec![0x09, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut stream: &[u8] = &stream;
        
        let expected_services = BitfieldServices::new(vec![
            SupportedServices::NodeNetworkLimited, 
            SupportedServices::NodeWitness, 
            SupportedServices::NodeNetwork
        ]);

        let services = BitfieldServices::deserialize(&mut stream)?;

        assert_eq!(expected_services, services);

        Ok(())
    }

}