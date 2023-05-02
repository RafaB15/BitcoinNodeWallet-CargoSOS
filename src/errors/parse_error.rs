use std::io::Error;
use std::net::AddrParseError;

pub enum ErroresParseo {
    ParseoIncorrectoDeInformacion,
    NoSuficientesValores,
    ParseoValorNoReconocido,
    CategoriaNoReconocida,
    ConfiguracionIncompleta,
}

impl std::convert::From<AddrParseError> for ErroresParseo {
    fn from(_: AddrParseError) -> Self {
        ErroresParseo::ParseoIncorrectoDeInformacion
    }
}

impl std::convert::From<Error> for ErroresParseo {
    fn from(_: Error) -> Self {
        ErroresParseo::ParseoIncorrectoDeInformacion
    }
}
