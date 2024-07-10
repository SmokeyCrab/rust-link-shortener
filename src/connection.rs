use std::net::IpAddr;

pub struct ClientContext {
    pub ip: IpAddr,
    pub port: u16,
}

impl ClientContext {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        ClientContext {
            ip: ip,
            port: port,
        }
    }
}
