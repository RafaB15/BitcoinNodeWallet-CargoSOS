#[cfg(test)]
mod test_integration {

    use ::cargosos_bitcoin::{
        configurations::settings::Settings,
        connections::{ibd_methods::IBDMethod, p2p_protocol::ProtocolVersionP2P},
    };
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test01_accept_valid_input() {
        let path = "tests/common/valid_configuration.txt";
        let configuration = Settings::new(path);

        let setting = Settings {
            dns_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            p2p_protocol_version: ProtocolVersionP2P::V70015,
            ibd_method: IBDMethod::HeaderFirst,
            filepath_log: "settings_implementation/log_prueba".to_string(),
        };

        assert_eq!(setting, configuration.unwrap());
    }
}
