use std::hash::Hash;
use std::io::{Error, ErrorKind};
use std::net::IpAddr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;


///Enum que representa el método de Initial Block Download que se va a utilizar
enum IBDMethod {
    BlocksFirst,
    HeaderFirst
}
///Enum que representa la versión del protocolo P2P que se va a utilizar
enum ProtocolVersionP2P {
    V70015,
    V70014,
    V70013,
    V70012,
    V70011,
    V70002,
    V70001,
    V60002,
    V60001,
    V60000,
    V31800,
    V31402,
    V311,
    V209,
    V106
}

///Struct que representa la configuración incial de nuestro programa
pub struct Settings {
    dns_address: IpAddr, //Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    p2p_protocol_version: ProtocolVersionP2P, // Es la versión del protocolo peer to peer que se planea utilizar
    ibd_method: IBDMethod, //El método usado para el initial blocks download
    filepath_log: String //La ruta al archivo en donde vamos a escribir el log
}

///Bloque de implementación de Settings
impl Settings {
    fn new(file_path: &str) -> Result<Self, Error> {
        let settings_file = File::open(file_path)?;
        let settings_reader = BufReader::new(settings_file);

        let settings_dictionary: HashMap<&str, &str> = HashMap::new();

        for line in settings_reader.lines() {
            let current_line = line?;

            let parts: Vec<&str> = current_line.split(':').collect();
        }

        Ok(Settings { dns_address: (), p2p_protocol_version: (), ibd_method: (), filepath_log: () })
    }
}