///Función asociada a Settings que crea un nuevo objeto en base al contenido de un archivo de texto
pub mod Config {

    use crate::configurations::{connection_config::ConnectionConfig, log_config::LogConfig};
    use crate::errors::parse_error::ErroresParseo;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::io::{Error, ErrorKind};

    pub type Configuraciones = (LogConfig, ConnectionConfig);

    const CONNECTIONS: &str = "connection";
    const LOGS: &str = "filepath_log";

    pub fn new(file_path: &str) -> Result<Configuraciones, ErroresParseo> {
        let settings_file: File = File::open(file_path)?;
        let settings_reader: BufReader<File> = BufReader::new(settings_file);

        let mut log_config: Option<LogConfig> = None;
        let mut connection_config: Option<ConnectionConfig> = None;

        let config_dictionary: HashMap<String, Vec<String>> =
            create_config_dictionary(settings_reader)?;

        for (key, value) in config_dictionary {
            let dictionary: HashMap<String, String> = create_property_value_dictionary(value)?;
            match key.as_str() {
                LOGS => log_config = Some(LogConfig::new(dictionary)?),
                CONNECTIONS => connection_config = Some(ConnectionConfig::new(dictionary)?),
                _ => return Err(ErroresParseo::CategoriaNoReconocida),
            }
        }

        if let (Some(log_config), Some(connection_config)) = (log_config, connection_config) {
            Ok((log_config, connection_config))
        } else {
            Err(ErroresParseo::ConfiguracionIncompleta)
        }
    }

    fn create_config_dictionary(
        settings_reader: BufReader<File>,
    ) -> Result<HashMap<String, Vec<String>>, ErroresParseo> {
        let config_dictionary: HashMap<String, Vec<String>> = HashMap::new();
        let mut text: Vec<String> = Vec::new();

        for line in settings_reader.lines() {
            let current_line = line?;
            text.push(current_line);
        }

        Ok(config_dictionary)
    }

    fn create_property_value_dictionary(
        file_files: Vec<String>,
    ) -> Result<HashMap<String, String>, Error> {
        let mut config_dictionary: HashMap<String, String> = HashMap::new();

        for line in file_files {
            let (key, value) = slit_linea(line)?;

            if config_dictionary.contains_key(&key) {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "One of the fields present is specified more than once.",
                ));
            }

            config_dictionary.insert(key, value);
        }

        Ok(config_dictionary)
    }

    fn slit_linea(file_line: String) -> Result<(String, String), Error> {
        let mut split_line = file_line.split(':');

        let (key, value) = match (split_line.next(), split_line.next()) {
            (Some(key), Some(value)) => (key, value),
            _ => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "One of the lines in the file does not have the correct format. The correct format is <field>:<value>",
                ))
            }
        };

        Ok((key.trim().to_string(), value.trim().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test01_accept_valid_input() {
        let path = "tests/common/valid_configuration.txt";
        let configuration = Config::new(path);

        let setting = Config {
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
}
