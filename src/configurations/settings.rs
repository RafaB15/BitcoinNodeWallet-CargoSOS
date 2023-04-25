use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Error, ErrorKind};
use std::net::IpAddr;

#[derive(Debug)]
///Struct que representa la configuración incial de nuestro programa
pub struct Settings {
    ///Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    pub dns_address: IpAddr,
    /// Es la versión del protocolo peer to peer que se planea utilizar
    pub p2p_protocol_version: ProtocolVersionP2P,
    ///El método usado para el initial blocks download
    pub ibd_method: IBDMethod,
    ///La ruta al archivo en donde vamos a escribir el log
    pub filepath_log: String,
}

///Bloque de implementación de Settings
impl Settings {
    ///Función asociada a Settings que crea un nuevo objeto en base al contenido de un archivo de texto
    pub fn new(file_path: &str) -> Result<Self, Error> {
        let settings_dictionary = Self::create_setting_dictionary(file_path)?;

        let dns_address: IpAddr = match settings_dictionary["dns_address"].parse::<IpAddr>() {
            Ok(address) => address,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "The IP address provided for the DNS server is not valid.",
                ))
            }
        };

        let p2p_protocol_version =
            settings_dictionary["p2p_protocol_version"].parse::<ProtocolVersionP2P>()?;
        let ibd_method = settings_dictionary["ibd_method"].parse::<IBDMethod>()?;

        let filepath_log = settings_dictionary["filepath_log"].clone();

        Ok(Settings {
            dns_address,
            p2p_protocol_version,
            ibd_method,
            filepath_log,
        })
    }

    ///Crea un HashMap con los campos del struct como llaves y el contenido como valores
    fn create_setting_dictionary(file_path: &str) -> Result<HashMap<String, String>, Error> {
        let settings_file = File::open(file_path)?;
        let settings_reader = BufReader::new(settings_file);

        let mut settings_dictionary: HashMap<String, String> = HashMap::new();

        for line in settings_reader.lines() {
            let current_line = line?;
            Self::read_line_config(&current_line, &mut settings_dictionary)?;
        }

        match settings_dictionary.contains_key("dns_address") && 
              settings_dictionary.contains_key("p2p_protocol_version") &&
              settings_dictionary.contains_key("ibd_method") &&
              settings_dictionary.contains_key("filepath_log") {
                true => Ok(settings_dictionary),
                false => Err(Error::new(
                    ErrorKind::InvalidInput,
                    "One of the necessary fields is not present. Check documentation for a list of all necessary fields.",
                ))
              }
    }

    ///Lee el contenido de una línea del archivo de configuración y guarda los contenidos en un HashMap
    fn read_line_config(
        current_line: &str,
        settings_dictionary: &mut HashMap<String, String>,
    ) -> Result<(), Error> {
        let mut current_line_split = current_line.split(':');

        let (key, value) = match (current_line_split.next(), current_line_split.next()) {
            (Some(key), Some(value)) => (key, value),
            _ => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "One of the lines in the file does not have the correct format. The correct format is <field>:<value>",
                ))
            }
        };

        if settings_dictionary.contains_key(key) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "One of the fields present is specified more than once.",
            ));
        }

        settings_dictionary.insert(key.trim().to_string(), value.trim().to_string());
        Ok(())
    }
}

impl std::cmp::PartialEq for Settings {
    fn eq(&self, other: &Self) -> bool {
        self.dns_address == other.dns_address
            && self.p2p_protocol_version == other.p2p_protocol_version
            && self.ibd_method == other.ibd_method
            && self.filepath_log == other.filepath_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test01_accept_valid_input() {
        let path = "tests/common/valid_configuration.txt";
        let configuration = Settings::new(path);

        let setting = Settings {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            filepath_log: "tests/common/log_prueba.txt".to_string(),
        };

        assert_eq!(setting, configuration.unwrap());
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let path = "tests/common/configuration_with_empty_spaces.txt";
        let configuration = Settings::new(path);

        let setting = Settings {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            filepath_log: "tests/common/log_prueba.txt".to_string(),
        };

        assert_eq!(setting, configuration.unwrap());
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_fields() {
        let path = "tests/common/configuration_with_missing_field.txt";
        let configuration = Settings::new(path);
        assert_eq!(configuration.err().unwrap().to_string().as_str(), "One of the necessary fields is not present. Check documentation for a list of all necessary fields.");
    }

    #[test]
    fn test04_does_not_accept_input_with_missing_values() {
        let path = "tests/common/configuration_with_missing_value.txt";
        let configuration = Settings::new(path);
        assert_eq!(configuration.err().unwrap().to_string().as_str(), "One of the lines in the file does not have the correct format. The correct format is <field>:<value>");
    }

    #[test]
    fn test05_does_not_accept_input_with_invalid_ibd() {
        let path = "tests/common/configuration_with_invalid_ibd.txt";
        let configuration = Settings::new(path);
        assert_eq!(configuration.err().unwrap().to_string().as_str(), "The provided method for the initial block download is not valid.");
    }

    #[test]
    fn test06_does_not_accept_input_with_invalid_p2p_protocol_version() {
        let path = "tests/common/configuration_with_invalid_p2p_version.txt";
        let configuration = Settings::new(path);
        assert_eq!(configuration.err().unwrap().to_string().as_str(), "The provided version for the P2P protocol is not valid.");
    }

    #[test]
    fn test07_does_not_accept_input_with_invalid_ip_address() {
        let path = "tests/common/configuration_with_invalid_ip_address.txt";
        let configuration = Settings::new(path);
        assert_eq!(configuration.err().unwrap().to_string().as_str(), "The IP address provided for the DNS server is not valid.");
    }

    #[test]
    fn test08_does_not_accept_input_with_duplicate_value() {
        let path = "tests/common/configuration_with_duplicate_value.txt";
        let configuration = Settings::new(path);
        assert_eq!(configuration.err().unwrap().to_string().as_str(), "One of the fields present is specified more than once.");
    }

}