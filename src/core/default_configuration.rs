use std::net::{
    SocketAddr,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr
};

use crate::{
    error::Error,
    node::Node,
    config::Config
};

pub struct DefaultConfiguration {
    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>,

    storage_path: String,
    bootstrap_nodes: Vec<Node>
}

pub struct Builder<'a> {
    ipv4: Option<&'a str>,
    ipv6: Option<&'a str>,
    port: u16,

    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>
}

impl DefaultConfiguration {
    fn new(b: &mut Builder) -> Self {
        DefaultConfiguration {
            addr4: b.addr4.take(),
            addr6: b.addr6.take(),
            storage_path: "".to_string(),
            bootstrap_nodes: Vec::new(),
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

    fn bootstrap_nodes(&self) -> &[Node] {
        &self.bootstrap_nodes

    }
}

#[allow(dead_code)]
impl<'a> Builder<'a> {
    pub fn new() -> Builder<'a> {
        Builder {
            ipv4: None,
            ipv6: None,
            port: 0,

            addr4: None,
            addr6: None
        }
    }

    pub fn with_ipv4(&mut self, ipv4: &'a str) -> &mut Self {
        self.ipv4 = Some(ipv4); self
    }

    pub fn with_ipv6(&mut self, ipv6: &'a str) -> &mut Self {
        self.ipv6 = Some(ipv6); self
    }

    pub fn with_listening_port(&mut self, port: u16) -> &mut Self {
        self.port = port; self
    }

    pub fn with_storage_path(&mut self, _: &str) -> &mut Self {
        unimplemented!()
    }

    pub fn storage_path(&self) -> &str {
        unimplemented!()
    }

    pub fn add_bootstrap_node(&mut self, _: &Node) -> &mut Self {
        unimplemented!()
    }

    pub fn build(&mut self) -> Result<Box<dyn Config>, Error> {
        if self.port == 0 {
            return Err(Error::Argument(format!("Invalid port value {}", self.port)));
        }

        if self.ipv4.is_some() {
            let addr = match self.ipv4.unwrap().parse::<Ipv4Addr>() {
                Ok(_addr) => _addr,
                Err(e) => {
                    return Err(Error::Argument(format!("error: {}", e)));
                }
            };

            self.addr4 = Some(SocketAddr::new(IpAddr::V4(addr), self.port));
        }
        if self.ipv6.is_some() {
            let addr = match self.ipv6.unwrap().parse::<Ipv6Addr>() {
                Ok(_addr) => _addr,
                Err(e) => {
                    return Err(Error::Argument(format!("error: {}", e)));
                }
            };
            self.addr6 = Some(SocketAddr::new(IpAddr::V6(addr), self.port));
        }

        Ok(Box::new(DefaultConfiguration::new(self)) as Box<dyn Config>)
    }
}
