use std::net::SocketAddr;
use crate::node::Node;

pub trait Config {
    fn ipv4(&self) -> &Option<SocketAddr>;
    fn ipv6(&self) -> &Option<SocketAddr>;

    fn storage_path(&self) -> &str;
    fn bootstrap_nodes(&self) -> &[Node];
}
