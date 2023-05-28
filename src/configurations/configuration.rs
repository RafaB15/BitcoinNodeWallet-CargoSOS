/*pub mod config {

    use crate::configurations::{
        connection_config::ConnectionConfig, deserializable_structure::DeserializeStructure,
        error_configuration::ErrorConfiguration, log_config::LogConfig,
    };
    use std::collections::HashMap;
    use std::io::Read;

    pub type Configuraciones = (LogConfig, ConnectionConfig);

    const CONNECTION_CONFIG: &str = "Connection";
    const LOGS_CONFIG: &str = "Logs";

    /// Returns all the configurations given a readable value
    ///
    /// ### Errors
    ///  * `ErrorReadableError`: It will appear when there given readable gives an error when read
    ///  * `ErrorIncompleteConfiguration`: It will appear when there isn't a configuration at all
    ///  * `ErrorConfigurationNoFount`: It will appear when there isn't a structure with a given property name
    pub fn new<R: Read>(configuration: R) -> Result<Configuraciones, ErrorConfiguration> {
        let config_dictionary: HashMap<String, Vec<String>> =
            create_config_dictionary(configuration)?;

        let log_config: LogConfig = LogConfig::deserializar(LOGS_CONFIG, &config_dictionary)?;
        let connection_config: ConnectionConfig =
            ConnectionConfig::deserializar(CONNECTION_CONFIG, &config_dictionary)?;

        Ok((log_config, connection_config))
    }

    /// Returns the structure of the configurations
    ///
    /// ### Errors
    ///  * `ErrorReadableError`: It will appear when there given readable gives an error when read
    ///  * `ErrorIncompleteConfiguration`: It will appear when there isn't a configuration at all
    fn create_config_dictionary<R: Read>(
        mut settings_reader: R,
    ) -> Result<HashMap<String, Vec<String>>, ErrorConfiguration> {
        let mut config_dictionary: HashMap<String, Vec<String>> = HashMap::new();

        let mut full_text: String = String::new();
        let _ = match settings_reader.read_to_string(&mut full_text) {
            Ok(len) => len,
            _ => {
                return Err(ErrorConfiguration::ErrorReadableError);
            }
        };

        let text: Vec<String> = full_text
            .split('\n')
            .map(|valor| valor.to_string())
            .collect();

        let title_positions: Vec<usize> = find_titles(&text);
        if title_positions.is_empty() {
            return Err(ErrorConfiguration::ErrorIncompleteConfiguration);
        }

        let last_position: usize = text.len();
        for (i, position) in title_positions.clone().into_iter().enumerate() {
            let next_position = *title_positions.get(i + 1).unwrap_or(&last_position);

            let title: String = match text.get(position) {
                Some(titulo) => titulo.to_owned(),
                _ => {
                    return Err(ErrorConfiguration::ErrorFieldNotFound);
                }
            };

            let config_info: Vec<String> = text[position + 1..next_position].to_vec();
            config_dictionary.insert(title.trim().to_string(), config_info);
        }

        Ok(config_dictionary)
    }

    /// Return the position of the titles of every structure
    fn find_titles(text: &[String]) -> Vec<usize> {
        let mut positions: Vec<usize> = Vec::new();

        for (i, line) in text.iter().enumerate() {
            if line.contains('[') && line.contains(']') {
                positions.push(i);
            }
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::config;
    use crate::configurations::{
        connection_config::ConnectionConfig, error_configuration::ErrorConfiguration,
        log_config::LogConfig,
    };
    use crate::connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P};

    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test01_accept_valid_input() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst
            [Logs]
            filepath_log:log_test.txt"
            .as_bytes();
        let config_result = config::new(configuration);

        let config_log = LogConfig {
            filepath_log: "log_test.txt".to_string(),
        };

        let config_connection = ConnectionConfig {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
        };

        assert_eq!(Ok((config_log, config_connection)), config_result);
    }

    #[test]
    fn test02_accepts_input_with_empty_spaces() {
        let configuration = "[Connection]
            dns_address      :127.0.0.1
            p2p_protocol_version:  V70015
            ibd_method            :       HeaderFirst
            [Logs]
            filepath_log:         log_test.txt"
            .as_bytes();
        let config_result = config::new(configuration);

        let config_log = LogConfig {
            filepath_log: "log_test.txt".to_string(),
        };

        let config_connection = ConnectionConfig {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
        };

        assert_eq!(Ok((config_log, config_connection)), config_result);
    }

    #[test]
    fn test03_does_not_accept_input_with_missing_configuration() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst"
            .as_bytes();
        let config_result = config::new(configuration);

        assert_eq!(
            config_result,
            Err(ErrorConfiguration::ErrorConfigurationNotFound)
        );
    }

    #[test]
    fn test04_does_not_accept_input_with_missing_values() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:
            [Logs]
            filepath_log:tests/common/log_test.txt"
            .as_bytes();
        let config_result = config::new(configuration);

        assert_eq!(config_result, Err(ErrorConfiguration::ErrorCantParseValue));
    }

    #[test]
    fn test05_does_not_accept_input_with_invalid_ibd() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:ChismeFirst
            [Logs]
            filepath_log:tests/common/log_test.txt"
            .as_bytes();
        let config_result = config::new(configuration);

        assert_eq!(config_result, Err(ErrorConfiguration::ErrorCantParseValue));
    }

    #[test]
    fn test06_does_not_accept_input_with_invalid_p2p_protocol_version() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:JK2000
            ibd_method:HeaderFirst
            [Logs]
            filepath_log:tests/common/log_test.txt"
            .as_bytes();
        let config_result = config::new(configuration);

        assert_eq!(config_result, Err(ErrorConfiguration::ErrorCantParseValue));
    }

    #[test]
    fn test07_does_not_accept_input_with_invalid_ip_address() {
        let configuration = "[Connection]
            dns_address:No soy un address muy válido que digamos
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst
            [Logs]
            filepath_log:log_test.txt"
            .as_bytes();
        let config_result = config::new(configuration);

        assert_eq!(config_result, Err(ErrorConfiguration::ErrorCantParseValue));
    }

    #[test]
    fn test08_does_not_accept_input_with_duplicate_value() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst
            ibd_method:BlockFirst
            [Logs]
            filepath_log:log_test.txt"
            .as_bytes();
        let config_result = config::new(configuration);

        assert_eq!(
            config_result,
            Err(ErrorConfiguration::ErrorEncounterFieldMoreThanOnes)
        );
    }

    #[test]
    fn test09_does_not_accept_input_with_not_information() {
        let configuration = "".as_bytes();
        let config_result = config::new(configuration);

        assert_eq!(
            config_result,
            Err(ErrorConfiguration::ErrorIncompleteConfiguration)
        );
    }
}
*/