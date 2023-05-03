#[cfg(test)]
mod test_integration {

    use cargosos_bitcoin::configurations::{
        config::config,
        log_config::LogConfig, 
        connection_config::ConnectionConfig
    };
    use cargosos_bitcoin::connections::{
        ibd_methods::IBDMethod,
        p2p_protocol::ProtocolVersionP2P,
    };

    use std::net::{IpAddr, Ipv4Addr};
    use std::fs::File;
    use std::io::{Error, BufReader};

    #[test]
    fn test01_se_lee_correctamente_la_configuracion() -> Result<(), Error>  {
        let file_path = "tests/common/valid_configuration.txt";
        let settings_file = File::open(file_path)?;
        let configuration = BufReader::new(settings_file);
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

        Ok(())
    }
}
