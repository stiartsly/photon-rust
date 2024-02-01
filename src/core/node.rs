use std::fmt;
use std::net::{IpAddr, SocketAddr};

use crate::id::Id;
use crate::version;

pub(crate) trait Reachable {
    fn reachable(&self) -> bool { false }
    fn unreachable(&self) -> bool { false }
    fn set_reachable(&mut self, _: bool) {}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Node {
    id: Id,
    addr: SocketAddr,
    ver: i32,
}

impl Node {
    pub fn new(id: &Id, addr: &SocketAddr) -> Self {
        Node {
            id: id.clone(),
            addr: addr.clone(),
            ver: 0
        }
    }

    pub const fn ip(&self) -> IpAddr {
        self.addr.ip()
    }

    pub const fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn socket_addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn version(&self) -> i32 {
        self.ver
    }

    pub fn set_version(&mut self, version: i32) {
        self.ver = version
    }

    pub fn is_ipv4(&self) -> bool{
        match self.addr.ip() {
            IpAddr::V4(_) => true,
            _ => false,
        }
    }

    pub fn is_ipv6(&self) -> bool{
        match self.addr.ip() {
            IpAddr::V6(_) => true,
            _ => false,
        }
    }

    pub fn readable_version(&self) -> String {
        version::readable_version(self.ver)
    }

    pub fn matches(&self, other: &Node) -> bool {
        self.id == other.id || self.addr == other.addr
    }
}

impl Reachable for Node {
    // Use the default implementation of trait.
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
