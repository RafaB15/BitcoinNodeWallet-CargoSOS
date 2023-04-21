use std::net::IpAddr;

enum IBDMethod {
    BlocksFirst,
    HeaderFirst
}

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

pub struct Settings {
    dns_address: IpAddr, //Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    p2p_protocol_version: ProtocolVersionP2P, // Es la versión del protocolo peer to peer que se planea utilizar
    ibd_method: IBDMethod, //El método usado para el initial blocks download
    filepath_log: String //La ruta al archivo en donde vamos a escribir el log
}