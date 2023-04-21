use std::net::IpAddr;

enum IBDMethod {
    BlocksFirst,
    HeaderFirst
}

enum P2P_ProtocolVersion {
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
    IpAddr: DNS_address, //Es la dirección IP del DNS de donde obtendremos las IP addresses de otros nodos
    P2P_ProtocolVersion: p2p_protocol_version,  // Es la versión del protocolo peer to peer que se planea utilizar
    IBDMethod: ibd_method //El método usado para el initial blocks download
}