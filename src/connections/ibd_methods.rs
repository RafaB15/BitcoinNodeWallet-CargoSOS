use super::connection_error::ConnectionError;

#[derive(Debug, std::cmp::PartialEq)]
///Enum que representa el método de Initial Block Download que se va a utilizar
pub enum IBDMethod {
    BlocksFirst,
    HeaderFirst,
}
///Implementación del trait que permite hacer parse
impl std::str::FromStr for IBDMethod {
    type Err = ConnectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BlocksFirst" => Ok(IBDMethod::BlocksFirst),
            "HeaderFirst" => Ok(IBDMethod::HeaderFirst),
            _ => todo!(),
        }
    }
}
