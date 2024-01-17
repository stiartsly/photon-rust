use std::fmt;
use std::net::{IpAddr, SocketAddr};

use crate::id::Id;
use crate::version;

pub trait NodeInfo {
    fn id(&self) -> &Id;
    fn socket_addr(&self) -> &SocketAddr;
    fn version(&self) -> i32;

    fn is_ipv4(&self) -> bool;
    fn is_ipv6(&self) -> bool;

    fn with_version(&mut self, version: i32) -> &mut Self;
}

pub(crate) trait Visit {
    fn reachable(&self) -> bool;
    fn unreachable(&self) -> bool;

    fn with_reachable(&mut self, _: bool) -> &mut Self;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    id: Id,
    addr: SocketAddr,
    ver: i32,
}

impl Node {
    pub fn new(id: &Id, socket_addr: &SocketAddr) -> Self {
        Node {
            id: *id,
            addr: *socket_addr,
            ver: 0
        }
    }

    pub const fn ip(&self) -> IpAddr {
        self.addr.ip()
    }

    pub const fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn readable_version(&self) -> String {
        version::readable_version(self.ver)
    }

    pub fn matches(&self, other: &Node) -> bool {
        self.id == other.id || self.addr == other.addr
    }
}

impl NodeInfo for Node {
    fn id(&self) -> &Id {
        &self.id
    }

    fn socket_addr(&self) -> &SocketAddr {
        &self.addr
    }

    fn version(&self) -> i32 {
        self.ver
    }

    fn with_version(&mut self, version: i32) -> &mut Self {
        self.ver = version; self
    }

    fn is_ipv4(&self) -> bool{
        match self.addr.ip() {
            IpAddr::V4(_) => true,
            _ => false,
        }
    }

    fn is_ipv6(&self) -> bool{
        match self.addr.ip() {
            IpAddr::V6(_) => true,
            _ => false,
        }
    }
}

impl Visit for Node {
    fn reachable(&self) -> bool {
        false
    }

    fn unreachable(&self) -> bool {
        false
    }

    fn with_reachable(&mut self, _: bool) -> &mut Self {
        self
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}",
            self.id,
            self.addr,
            version::readable_version(self.ver)
        )?;
        Ok(())
    }
}
