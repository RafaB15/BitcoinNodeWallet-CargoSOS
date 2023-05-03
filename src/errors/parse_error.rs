use std::net::AddrParseError;
use std::io::Error;

pub enum ErroresParseo {
    ErrorParseoIncorrectoDeInformacion,
    ErrorNoSuficientesValores,
    ErrorParseoValorNoReconocido,
    ErrorCategoriaNoReconocida,
    ErrorConfiguracionIncompleta,
    ErrorNoHayCategorias,
    ErrorCategoriaAparareceMasDeUnaVez,
    ErrorFormatoIncorrecto,
}

impl std::convert::From<AddrParseError> for ErroresParseo {
    fn from(_: AddrParseError) -> Self {
        ErroresParseo::ErrorParseoIncorrectoDeInformacion
    }
}

impl std::convert::From<Error> for ErroresParseo {
    fn from(_: Error) -> Self {
        ErroresParseo::ErrorParseoIncorrectoDeInformacion
    }
}
