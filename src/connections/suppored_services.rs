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
