use super::error_connection::ErrorConnection;

#[derive(Debug, std::cmp::PartialEq, Copy, Clone)]
///Enum que representa el método de Initial Block Download que se va a utilizar
pub enum IBDMethod {
    BlocksFirst,
    HeaderFirst,
}
///Implementación del trait que permite hacer parse
impl std::str::FromStr for IBDMethod {
    type Err = ErrorConnection;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BlocksFirst" => Ok(IBDMethod::BlocksFirst),
            "HeaderFirst" => Ok(IBDMethod::HeaderFirst),
            _ => Err(ErrorConnection::ErrorInvalidInputParse),
        }
    }
}
