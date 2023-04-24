use std::io::{Error, ErrorKind};
#[derive(Debug)]
///Enum que representa el método de Initial Block Download que se va a utilizar
pub enum IBDMethod {
    BlocksFirst,
    HeaderFirst
}
///Implementación del trait que permite hacer parse
impl std::str::FromStr for IBDMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BlocksFirst" => Ok(IBDMethod::BlocksFirst),
            "HeaderFirst" => Ok(IBDMethod::HeaderFirst),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "El método proporcionado para la descarga inicial de bloques no es válido.",
            ))
        }
    }
}