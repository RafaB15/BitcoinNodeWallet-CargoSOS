///Función asociada a Settings que crea un nuevo objeto en base al contenido de un archivo de texto
pub mod config {

    use crate::configurations::{
        estructura_deserializable::EstructuraDeserializable,
        connection_config::ConnectionConfig, 
        log_config::LogConfig,
        parse_error::ErroresParseo,
    };
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    pub type Configuraciones = (LogConfig, ConnectionConfig);

    pub fn new(file_path: &str) -> Result<Configuraciones, ErroresParseo> {
        let settings_file: File = match File::open(file_path) {
            Ok(file) => file,
            _ => { return Err(ErroresParseo::ErrorNoExisteArchivo); }
        };
        let settings_reader = BufReader::new(settings_file);

        let config_dictionary: HashMap<String, Vec<String>> =
            create_config_dictionary(settings_reader)?;

        let log_config: LogConfig = LogConfig::default();
        let connection_config: ConnectionConfig = ConnectionConfig::default();

        log_config.deserializar(&config_dictionary)?;
        connection_config.deserializar(&config_dictionary)?;

        Ok((log_config, connection_config))
    }

    fn create_config_dictionary(
        settings_reader: BufReader<File>,
    ) -> Result<HashMap<String, Vec<String>>, ErroresParseo> {
        let mut config_dictionary: HashMap<String, Vec<String>> = HashMap::new();
        let mut text: Vec<String> = Vec::new();

        for line in settings_reader.lines() {
            let current_line = line?;
            text.push(current_line);
        }

        let ubicacion_titulos: Vec<usize> = encontrar_titulos(&text);
        if ubicacion_titulos.len() == 0 {
            return Err(ErroresParseo::ErrorNoHayCategoria);
        }

        let ubicacion_final: usize = text.len();
        for (i, ubicacion) in ubicacion_titulos.clone().into_iter().enumerate() {
            let ubicacion_siguiente = ubicacion_titulos.get(i + 1).unwrap_or(&ubicacion_final);
            
            let titulo: String = match text.get(ubicacion) {
                Some(titulo) => titulo.to_owned(),
                _ => { return Err(ErroresParseo::ErrorNoHayCategoria); }
            };

            let informacion: Vec<String> = text[ubicacion + 1..ubicacion_siguiente - 1].to_vec(); 
            config_dictionary.insert(titulo.trim().to_string(), informacion);    
        }

        Ok(config_dictionary)
    }

    fn encontrar_titulos(file_lines: &Vec<String>) -> Vec<usize> {
        let mut posiciones: Vec<usize> = Vec::new();

        for (i, line) in file_lines.iter().enumerate() {

            if line.contains("[") && line.contains("]") {
                posiciones.push(i);
            }
        }

        posiciones
    }
}

#[cfg(test)]
mod tests {
    use super::config;
    use crate::configurations::{
        parse_error::ErroresParseo, 
        log_config::LogConfig, 
        connection_config::ConnectionConfig
    };
    use crate::connections::{
        ibd_methods::IBDMethod,
        p2p_protocol::ProtocolVersionP2P,
    };

    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test00_file_not_existing() {
        let path = "tests/common/random_name.txt";
        let resultado_config = config::new(path);

        assert_eq!(Err(ErroresParseo::ErrorNoExisteArchivo), resultado_config);
    }

    #[test]
    fn test01_accept_valid_input() {
        let path = "tests/common/valid_configuration.txt";
        let resultado_config = config::new(path);

        let config_log = LogConfig {
            filepath_log: "tests/common/log_prueba.txt".to_string(),
        };

        let config_connection = ConnectionConfig {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
        };

        assert_eq!(Ok((config_log, config_connection)), resultado_config);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let path = "tests/common/configuration_with_empty_spaces.txt";
        let resultado_config = config::new(path);

        let config_log = LogConfig {
            filepath_log: "tests/common/log_prueba.txt".to_string(),
        };

        let config_connection = ConnectionConfig {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
        };

        assert_eq!(Ok((config_log, config_connection)), resultado_config);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_fields() {
        let path = "tests/common/configuration_with_missing_field.txt";
        let resultado_config = config::new(path);

        assert_eq!(resultado_config, Err(ErroresParseo::ErrorNoHayCategoria));
    }

    /*
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
        assert_eq!(
            configuration.err().unwrap().to_string().as_str(),
            "The provided method for the initial block download is not valid."
        );
    }

    #[test]
    fn test06_does_not_accept_input_with_invalid_p2p_protocol_version() {
        let path = "tests/common/configuration_with_invalid_p2p_version.txt";
        let configuration = Settings::new(path);
        assert_eq!(
            configuration.err().unwrap().to_string().as_str(),
            "The provided version for the P2P protocol is not valid."
        );
    }

    #[test]
    fn test07_does_not_accept_input_with_invalid_ip_address() {
        let path = "tests/common/configuration_with_invalid_ip_address.txt";
        let configuration = Settings::new(path);
        assert_eq!(
            configuration.err().unwrap().to_string().as_str(),
            "The IP address provided for the DNS server is not valid."
        );
    }

    #[test]
    fn test08_does_not_accept_input_with_duplicate_value() {
        let path = "tests/common/configuration_with_duplicate_value.txt";
        let configuration = Settings::new(path);
        assert_eq!(
            configuration.err().unwrap().to_string().as_str(),
            "One of the fields present is specified more than once."
        );
    }

     */
}
