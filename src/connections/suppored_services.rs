use super::connection_error::ConnectionError;

#[derive(Debug, std::cmp::PartialEq)]
///
pub enum SupportedServices {
    Unname,
    NodeNetwork,
    NodeGetUTXO,
    NodeBloom,
    NodeWitness,
    NodeXThin,
    NodeNetworkLimited,
}

///Implementación del trait que permite hacer parse
impl std::str::FromStr for SupportedServices {
    type Err = ConnectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Err(ConnectionError::ErrorInvalidInputParse)
    }

}

impl std::convert::TryFrom<i64> for SupportedServices {
    type Error = ConnectionError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SupportedServices::Unname),
            1 => Ok(SupportedServices::NodeNetwork),
            2 => Ok(SupportedServices::NodeGetUTXO),
            4 => Ok(SupportedServices::NodeBloom),
            8 => Ok(SupportedServices::NodeWitness),
            10 => Ok(SupportedServices::NodeXThin),
            400 => Ok(SupportedServices::NodeNetworkLimited),
            _ => return Err(ConnectionError::ErrorInvalidInputParse),
        }
    }
}

impl std::convert::TryInto<i64> for SupportedServices {
    type Error = ConnectionError;

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            SupportedServices::Unname => Ok(0),
            SupportedServices::NodeNetwork => Ok(1),
            SupportedServices::NodeGetUTXO => Ok(2),
            SupportedServices::NodeBloom => Ok(4),
            SupportedServices::NodeWitness => Ok(8),
            SupportedServices::NodeXThin => Ok(10),
            SupportedServices::NodeNetworkLimited => Ok(400),
        }
    }
}
