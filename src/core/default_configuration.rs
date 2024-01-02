use std::option::{Option};
use std::net::SocketAddr;
use crate::nodeinfo::NodeInfo;
use crate::config::Config;

#[allow(dead_code)]
pub(crate) struct DefaultConfiguration {
    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>,

    storage_path: String,
    bootstrap_nodes: Vec<NodeInfo>
}

#[allow(dead_code)]
impl DefaultConfiguration {
    pub(crate) fn new(ipv4: Option<&SocketAddr>, ipv6: Option<&SocketAddr>) -> Self {
        DefaultConfiguration {
            addr4: ipv4.map(|&addr| addr),
            addr6: ipv6.map(|&addr| addr),
            storage_path: String::new(),
            bootstrap_nodes: Vec::new()
        }
    }
}

impl Config for DefaultConfiguration {
    fn ipv4(&self) -> &Option<SocketAddr> {
        &self.addr4
    }

    fn ipv6(&self) -> &Option<SocketAddr> {
        &self.addr6
    }

    fn storage_path(&self) -> &str {
        &self.storage_path
    }

    fn bootstrap_nodes(&self) -> &[NodeInfo] {
        &self.bootstrap_nodes

    }
}