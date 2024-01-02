use std::fmt;
use std::net::{IpAddr, SocketAddr};
use crate::id::Id;

pub trait NodeInfoTrait {
    fn id(&self) -> &Id;
    fn socket_addr(&self) -> &SocketAddr;

    fn version(&self) -> i32;

    fn set_version(&mut self, version: i32);
    fn is_ipv4(&self) -> bool;
    fn is_ipv6(&self) -> bool;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NodeInfo {
    node_id: Id,
    socket_addr: SocketAddr,
    version: i32,
}

impl NodeInfo {
    pub fn new(id: &Id, socket_addr: &SocketAddr) -> Self {
        NodeInfo {
            node_id: *id,
            socket_addr: *socket_addr,
            version: 0
        }
    }

    pub const fn ip(&self) -> IpAddr {
        self.socket_addr.ip()
    }

    pub const fn port(&self) -> u16 {
        self.socket_addr.port()
    }

    pub const fn readable_version() -> String {
        unimplemented!();
    }

    pub fn matches(&self, other: &NodeInfo) -> bool {
        self.node_id == other.node_id || self.socket_addr == other.socket_addr
    }
}

impl NodeInfoTrait for NodeInfo {
    fn id(&self) -> &Id {
        &self.node_id
    }

    fn socket_addr(&self) -> &SocketAddr {
        &self.socket_addr
    }

    fn version(&self) -> i32 {
        self.version
    }

    fn set_version(&mut self, version: i32) {
        self.version = version;
    }

    fn is_ipv4(&self) -> bool{
        match self.socket_addr.ip() {
            IpAddr::V4(_) => true,
            _ => false,
        }
    }

    fn is_ipv6(&self) -> bool{
        match self.socket_addr.ip() {
            IpAddr::V6(_) => true,
            _ => false,
        }
    }
}

pub(crate) trait Accessibility {
    fn reachable(&self) -> bool;
    fn unreachable(&self) -> bool;

    fn set_reachable(&mut self, _: bool);
}

impl fmt::Display for NodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},", self.node_id)?;
        write!(f, "{},", self.socket_addr)?;
        write!(f, "{}", self.version)?;
        Ok(())
    }
}
