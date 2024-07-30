use std::net::SocketAddr;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Ipv4 = 4,
    Ipv6 = 6,
}

impl Network {
    pub fn of(addr: &SocketAddr) -> Self {
        match addr.is_ipv4() {
            true => Network::Ipv4,
            false => Network::Ipv6,
        }
    }
}
