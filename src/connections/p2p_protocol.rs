use super::connection_error::ConnectionError;

#[derive(Debug, std::cmp::PartialEq)]
///Enum que representa la versión del protocolo P2P que se va a utilizar
pub enum ProtocolVersionP2P {
    V70015,
    V70014,
    V70013,
    V70012,
    V70011,
    V70002,
    V70001,
    V60002,
    V60001,
    V60000,
    V31800,
    V31402,
    V311,
    V209,
    V106,
}
///Implementación del trait que permite hacer parse
impl std::str::FromStr for ProtocolVersionP2P {
    type Err = ConnectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "V70015" => Ok(ProtocolVersionP2P::V70015),
            "V70014" => Ok(ProtocolVersionP2P::V70014),
            "V70013" => Ok(ProtocolVersionP2P::V70013),
            "V70012" => Ok(ProtocolVersionP2P::V70012),
            "V70011" => Ok(ProtocolVersionP2P::V70011),
            "V70002" => Ok(ProtocolVersionP2P::V70002),
            "V70001" => Ok(ProtocolVersionP2P::V70001),
            "V60002" => Ok(ProtocolVersionP2P::V60002),
            "V60001" => Ok(ProtocolVersionP2P::V60001),
            "V60000" => Ok(ProtocolVersionP2P::V60000),
            "V31800" => Ok(ProtocolVersionP2P::V31800),
            "V31402" => Ok(ProtocolVersionP2P::V31402),
            "V311" => Ok(ProtocolVersionP2P::V311),
            "V209" => Ok(ProtocolVersionP2P::V209),
            "V106" => Ok(ProtocolVersionP2P::V106),
            _ => todo!(),
        }
    }
}
