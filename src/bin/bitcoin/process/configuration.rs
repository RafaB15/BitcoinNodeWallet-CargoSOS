use cargosos_bitcoin::configurations::{
    client_config::ClientConfig,
    connection_config::ConnectionConfig,
    download_config::DownloadConfig,
    error_configuration::ErrorConfiguration,
    log_config::LogConfig,
    mode_config::ModeConfig,
    parsable::{parse_structure, Parsable},
    save_config::SaveConfig,
    server_config::ServerConfig,
    ui_config::UIConfig,
};

use std::io::Read;

type Configurations = (
    LogConfig,
    ConnectionConfig,
    DownloadConfig,
    SaveConfig,
    UIConfig,
    ModeConfig,
);

const CONNECTION_CONFIG: &str = "Connection";
const LOGS_CONFIG: &str = "Logs";
const DOWNLOAD_CONFIG: &str = "Download";
const SAVE_CONFIG: &str = "Save";
const UI_CONFIG: &str = "UI";
const UI_SERVER: &str = "Server";
const UI_CLIENT: &str = "Client";

/// Represents all the configuration needed to run the program
#[derive(Debug, Clone)]
pub struct Configuration {
    pub log_config: LogConfig,
    pub connection_config: ConnectionConfig,
    pub download_config: DownloadConfig,
    pub save_config: SaveConfig,
    pub ui_config: UIConfig,
    pub mode_config: ModeConfig,
}

impl Configuration {
    /// Creates a new configuration from a stream file
    ///
    /// ### Error
    ///  * `ErrorConfiguration::ValueNotFound`: It will appear when the value is not found
    ///  * `ErrorConfiguration::ErrorIncompleteConfiguration`: It will appear when the configuration cannot be accessed
    ///  * `ErrorConfiguration::ErrorCantParseValue`: It will appear when the value cannot be parsed
    pub fn new<R: Read>(mut stream: R) -> Result<Self, ErrorConfiguration> {
        let mut value = String::new();
        if stream.read_to_string(&mut value).is_err() {
            return Err(ErrorConfiguration::ValueNotFound);
        }

        let map = parse_structure(value)?;

        let possible_server_config = Option::<ServerConfig>::parse(UI_SERVER, &map)?;
        let possible_client_config = Option::<ClientConfig>::parse(UI_CLIENT, &map)?;

        let mode_config = match (possible_server_config, possible_client_config) {
            (None, Some(client_config)) => ModeConfig::Client(client_config),
            (Some(server_config), _) => ModeConfig::Server(server_config),
            _ => return Err(ErrorConfiguration::ErrorIncompleteConfiguration),
        };

        Ok(Configuration {
            log_config: LogConfig::parse(LOGS_CONFIG, &map)?,
            connection_config: ConnectionConfig::parse(CONNECTION_CONFIG, &map)?,
            download_config: DownloadConfig::parse(DOWNLOAD_CONFIG, &map)?,
            save_config: SaveConfig::parse(SAVE_CONFIG, &map)?,
            ui_config: UIConfig::parse(UI_CONFIG, &map)?,
            mode_config,
        })
    }

    /// Separates the configuration into its parts to handle them separately
    pub fn separate(self) -> Configurations {
        (
            self.log_config,
            self.connection_config,
            self.download_config,
            self.save_config,
            self.ui_config,
            self.mode_config,
        )
    }
}
