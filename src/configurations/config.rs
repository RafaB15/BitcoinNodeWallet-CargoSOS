///Función asociada a Settings que crea un nuevo objeto en base al contenido de un archivo de texto
pub mod config {

    use crate::configurations::{
        estructura_deserializable::EstructuraDeserializable,
        connection_config::ConnectionConfig, 
        log_config::LogConfig,
        parse_error::ErroresParseo,
    };
    use std::collections::HashMap;
    use std::io::Read;

    pub type Configuraciones = (LogConfig, ConnectionConfig);

    pub fn new<R: Read>(configuration: R) -> Result<Configuraciones, ErroresParseo> {
        let config_dictionary: HashMap<String, Vec<String>> =
            create_config_dictionary(configuration)?;

        let log_config: LogConfig = LogConfig::deserializar(&config_dictionary)?;
        let connection_config: ConnectionConfig = ConnectionConfig::deserializar(&config_dictionary)?;

        Ok((log_config, connection_config))
    }

    fn create_config_dictionary<R: Read>(
        mut settings_reader: R
    ) -> Result<HashMap<String, Vec<String>>, ErroresParseo> {
        let mut config_dictionary: HashMap<String, Vec<String>> = HashMap::new();
        
        let mut full_text: String = String::new();
        let _ = match settings_reader.read_to_string(&mut full_text) {
            Ok(len) => len,
            _ => { return Err(ErroresParseo::ErrorNoExisteArchivo); }
        };
        
        let text: Vec<String> = full_text.split('\n').map(|valor| valor.to_string()).collect();
        if text.len() <= 1 {
            return Err(ErroresParseo::ErrorNoHayCategoria);
        }

        let ubicacion_titulos: Vec<usize> = encontrar_titulos(&text);
        if ubicacion_titulos.is_empty() {
            return Err(ErroresParseo::ErrorNoHayCategoria);
        }

        let ultima_posicion: usize = text.len();
        for (i, ubicacion) in ubicacion_titulos.clone().into_iter().enumerate() {
            let ubicacion_siguiente = *ubicacion_titulos.get(i + 1).unwrap_or(&ultima_posicion);
            
            let titulo: String = match text.get(ubicacion) {
                Some(titulo) => titulo.to_owned(),
                _ => { return Err(ErroresParseo::ErrorNoHayCategoria); }
            };

            let informacion: Vec<String> = text[ubicacion + 1..ubicacion_siguiente].to_vec(); 
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
    fn test01_accept_valid_input() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst
            [Logs]
            filepath_log:log_prueba.txt".as_bytes();
        let resultado_config = config::new(configuration);

        let config_log = LogConfig {
            filepath_log: "log_prueba.txt".to_string(),
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
        let configuration = "[Connection]
            dns_address      :127.0.0.1
            p2p_protocol_version:  V70015
            ibd_method            :       HeaderFirst
            [Logs]
            filepath_log:         log_prueba.txt".as_bytes();
        let resultado_config = config::new(configuration);

        let config_log = LogConfig {
            filepath_log: "log_prueba.txt".to_string(),
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
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst
            [Logs]".as_bytes();
        let resultado_config = config::new(configuration);

        assert_eq!(resultado_config, Err(ErroresParseo::ErrorConfiguracionIncompleta));
    }

    #[test]
    fn test04_does_not_accept_input_with_missing_values() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:
            [Logs]
            filepath_log:tests/common/log_prueba.txt".as_bytes();
        let resultado_config = config::new(configuration);

        assert_eq!(resultado_config, Err(ErroresParseo::ErrorConfiguracionIncompleta));
    }
    
    #[test]
    fn test05_does_not_accept_input_with_invalid_ibd() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:ChismeFirst
            [Logs]
            filepath_log:tests/common/log_prueba.txt".as_bytes();
        let resultado_config = config::new(configuration);

        assert_eq!(resultado_config, Err(ErroresParseo::ErrorConfiguracionIncompleta));
    }
    
    #[test]
    fn test06_does_not_accept_input_with_invalid_p2p_protocol_version() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:JK2000
            ibd_method:HeaderFirst
            [Logs]
            filepath_log:tests/common/log_prueba.txt".as_bytes();
        let resultado_config = config::new(configuration);

        assert_eq!(resultado_config, Err(ErroresParseo::ErrorConfiguracionIncompleta));
    }
    
    #[test]
    fn test07_does_not_accept_input_with_invalid_ip_address() {
        let configuration = "[Connection]
            dns_address:No soy un address muy válido que digamos
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst
            [Logs]
            filepath_log:log_prueba.txt".as_bytes();
        let resultado_config = config::new(configuration);

        assert_eq!(resultado_config, Err(ErroresParseo::ErrorConfiguracionIncompleta));
    }
             
    
    #[test]
    fn test08_does_not_accept_input_with_duplicate_value() {
        let configuration = "[Connection]
            dns_address:127.0.0.1
            p2p_protocol_version:V70015
            ibd_method:HeaderFirst
            ibd_method:BlockFirst
            [Logs]
            filepath_log:log_prueba.txt".as_bytes();
        let resultado_config = config::new(configuration);

        assert_eq!(resultado_config, Err(ErroresParseo::ErrorCategoriaAparareceMasDeUnaVez));
    }    
}
