use std::net::{IpAddr, SocketAddr};
use crate::id::Id;

pub struct NodeInfo {
    node_id: Id,
    sockaddr: SocketAddr,
    version: i32,
}

impl NodeInfo {
    /*fn new(id: Id, ip: &str, port: u16) -> NodeInfo {
        let ip_addr: IpAddr = ip.parse().unwrap();
        let sockaddr = SocketAddr::new(ip_addr, port);
        NodeInfo {
            node_id: id,
            sockaddr,
            version: 0,
        }
    }*/

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
}
