use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig,
    download_config::DownloadConfig,
    error_configuration::ErrorConfiguration,
    log_config::LogConfig,
    parsable::{parse_structure, Parsable},
    save_config::SaveConfig,
    ui_config::UIConfig,
};

use std::io::Read;

type Configurations = (
    LogConfig,
    ConnectionConfig,
    DownloadConfig,
    SaveConfig,
    UIConfig,
);

const CONNECTION_CONFIG: &str = "Connection";
const LOGS_CONFIG: &str = "Logs";
const DOWNLOAD_CONFIG: &str = "Download";
const SAVE_CONFIG: &str = "Save";
const UI_CONFIG: &str = "UI";

/// Represents all the configuration needed to run the program
#[derive(Debug, Clone)]
pub struct Configuration {
    pub log_config: LogConfig,
    pub connection_config: ConnectionConfig,
    pub download_config: DownloadConfig,
    pub save_config: SaveConfig,
    pub ui_config: UIConfig,
}

impl Configuration {
    /// Creates a new configuration from a stream file
    ///
    /// ### Error
    ///  *
    pub fn new<R: Read>(mut stream: R) -> Result<Self, ErrorConfiguration> {
        let mut value = String::new();
        if stream.read_to_string(&mut value).is_err() {
            return Err(ErrorConfiguration::ValueNotFound);
        }

        let map = parse_structure(value)?;
        Ok(Configuration {
            log_config: LogConfig::parse(LOGS_CONFIG, &map)?,
            connection_config: ConnectionConfig::parse(CONNECTION_CONFIG, &map)?,
            download_config: DownloadConfig::parse(DOWNLOAD_CONFIG, &map)?,
            save_config: SaveConfig::parse(SAVE_CONFIG, &map)?,
            ui_config: UIConfig::parse(UI_CONFIG, &map)?,
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
        )
    }
}
