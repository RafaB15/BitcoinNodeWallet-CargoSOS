use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};
use super::parse_error::ErroresParseo;
use std::collections::HashMap;
use std::str::FromStr;
use std::net::IpAddr;

pub trait Deserializable<'d> {
    type Valor : FromStr;

    fn deserializar(&self, settings_dictionary: &'d HashMap<String, String>) -> Result<Self::Valor, ErroresParseo> {
        let nombre: &str = stringify!(self);

        if let Some(valor) = settings_dictionary.get(nombre) {
            match valor.parse::<Self::Valor>() {
                Ok(resultado) => Ok(resultado),
                _ => return Err(ErroresParseo::ErrorConfiguracionIncompleta),
            }
        } else {
            Err(ErroresParseo::ErrorConfiguracionIncompleta)
        }
    }
}

impl<'d> Deserializable<'d> for String {
    type Valor = String;
}

impl<'d> Deserializable<'d> for IBDMethod {
    type Valor = IBDMethod;
}

impl<'d> Deserializable<'d> for ProtocolVersionP2P {
    type Valor = ProtocolVersionP2P;
}

impl<'d> Deserializable<'d> for IpAddr {
    type Valor = ProtocolVersionP2P;
}

