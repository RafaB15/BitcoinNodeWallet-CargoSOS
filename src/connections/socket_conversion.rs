use std::net::{
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr
};

pub fn socket_to_ipv6_port(socket_addr: &SocketAddr) -> (Ipv6Addr, u16) {
    let ip = match socket_addr.ip() {
        IpAddr::V4(v4) => Ipv4Addr::to_ipv6_mapped(&v4),
        IpAddr::V6(v6) => v6,
    };
    (ip, socket_addr.port())
}