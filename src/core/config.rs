use std::net::SocketAddr;
use crate::node_info::NodeInfo;

pub trait Config {
    fn addr4(&self) -> &Option<SocketAddr>;
    fn addr6(&self) -> &Option<SocketAddr>;

    fn storage_path(&self) -> &str;
    fn bootstrap_nodes(&self) -> &[NodeInfo];

    fn dump(&self);
}
