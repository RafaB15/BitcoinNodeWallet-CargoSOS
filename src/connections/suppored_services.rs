use super::connection_error::ConnectionError;

#[derive(Debug, std::cmp::PartialEq)]
///
pub enum SupportedServices {
    UNNAME,
    NODE_NETWORK,
    NODE_GETUTXO,
    NODE_BLOOM,
    NODE_WITNESS,
    NODE_XTHIN,
    NODE_NETWORK_LIMITED,
}

///Implementación del trait que permite hacer parse
impl std::str::FromStr for SupportedServices {
    type Err = ConnectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!();
    }
}
