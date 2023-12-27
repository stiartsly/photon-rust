use std::net::{IpAddr, SocketAddr};
use crate::id::Id;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeInfo {
    node_id: Id,
    sockaddr: SocketAddr,
    version: i32,
}

impl NodeInfo {
    pub fn new(id: &Id, socket_addr: &SocketAddr) -> Self {
        NodeInfo {
            node_id: *id,
            sockaddr: *socket_addr,
            version: 0
        }
    }

    pub fn id(&self) -> &Id {
        &self.node_id
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.sockaddr
    }

    pub fn ip(&self) -> IpAddr {
        self.sockaddr.ip()
    }

    pub fn port(&self) -> u16 {
        self.sockaddr.port()
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn is_ipv4(&self) -> bool{
        match self.sockaddr.ip() {
            IpAddr::V4(_) => true,
            _ => false,
        }
    }

    pub fn is_ipv6(&self) -> bool{
        match self.sockaddr.ip() {
            IpAddr::V6(_) => true,
            _ => false,
        }
    }
}
