use std::io::{Error, ErrorKind};
use std::net::IpAddr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use crate::p2p_protocol::ProtocolVersionP2P;
use crate::ibd_methods::IBDMethod;

///Struct que representa la configuración incial de nuestro programa
pub struct Settings {
    dns_address: IpAddr, //Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    p2p_protocol_version: ProtocolVersionP2P, // Es la versión del protocolo peer to peer que se planea utilizar
    ibd_method: IBDMethod, //El método usado para el initial blocks download
    filepath_log: String //La ruta al archivo en donde vamos a escribir el log
}

///Bloque de implementación de Settings
impl Settings {

    pub fn new(file_path: &str) -> Result<Self, Error> {
        let settings_dictionary = Self::create_setting_dictionary(file_path)?;

        let dns_address: IpAddr = match settings_dictionary["dns_address"].parse::<IpAddr>() {
            Ok(address) => address,
            Err(_) => return Err(Error::new(
                ErrorKind::InvalidInput,
                "La dirección IP proporcionada para el servidor DNS no es válida",
            ))
        };

        let p2p_protocol_version = settings_dictionary["p2p_protocol_version"].parse::<ProtocolVersionP2P>()?;
        let ibd_method = settings_dictionary["ibd_method"].parse::<IBDMethod>()?;

        let filepath_log = settings_dictionary["filepath_log"].clone();

        Ok(Settings {dns_address, p2p_protocol_version, ibd_method, filepath_log})
    }

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
                    "Uno de los campos necesarios no está presente. Revisar documentación para ver lista de campos necesarios.",
                ))
              }
    }

    fn read_line_config(current_line: &str, settings_dictionary: &mut HashMap<String, String>) -> Result<(), Error> {
        let mut current_line_split = current_line.split(":");

        let (key, value) = match (current_line_split.next(), current_line_split.next()) {
            (Some(key), Some(value)) => (key, value),
            _ => return Err(Error::new(
                ErrorKind::NotFound,
                "Las líneas del archivo no tienen el formato correcto",
            ))
        };

        if settings_dictionary.contains_key(key) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Uno de los campos ingresados está especificado más de una vez",
            ));
        }

        settings_dictionary.insert(key.trim().to_string(), value.trim().to_string());
        Ok(())
    }

}