use std::env;
use std::fmt;
use std::net::{
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr
};

use crate::{
    NodeInfo,
    config::Config,
    error::Error
};

pub struct Builder<'a> {
    ipv4: Option<&'a str>,
    ipv6: Option<&'a str>,
    port: u16,

    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>,
    storage_path: String,
    bootstrap_nodes: Vec<NodeInfo>,
}

impl<'a> Builder<'a> {
    pub fn new() -> Builder<'a> {
        let path = match env::var("HOME") {
            Ok(v) => v,
            _ => ".".to_string(),
        };

        Self {
            ipv4: None,
            ipv6: None,
            port: 0,

            addr4: None,
            addr6: None,

            storage_path: path,
            bootstrap_nodes: Vec::new(),
        }
    }

    pub fn with_ipv4(&mut self, input: &'a str) -> &mut Self {
        self.ipv4 = Some(input);
        self
    }

    pub fn with_ipv6(&mut self, input: &'a str) -> &mut Self {
        self.ipv6 = Some(input);
        self
    }

    pub fn with_listening_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn with_storage_path(&mut self, input: &'a str) -> &mut Self {
        if input.starts_with("~") {
            self.storage_path += &input[1..];
        } else {
            self.storage_path = input.to_string();
        }
        self
    }

    pub fn storage_path(&self) -> &str {
        &self.storage_path
    }

    pub fn add_bootstrap_node(&mut self, node: &NodeInfo) -> &mut Self {
        self.bootstrap_nodes.push(node.clone());
        self
    }

    pub fn add_bootstrap_nodes(&mut self, nodes: &[NodeInfo]) -> &mut Self {
        for item in nodes.iter() {
            self.bootstrap_nodes.push(item.clone())
        }
        self
    }

    pub fn load(&mut self, _: &str) -> &mut Self {
        unimplemented!()
    }

    pub fn check_valid(&self) -> Result<bool, Error> {
        if self.port == 0 {
            return Err(Error::Argument(format!("error: port can't be 0")));
        }

        if let Some(addr) = self.ipv4.as_ref() {
            addr.parse::<Ipv4Addr>().map_err(|e| {
                return Error::Argument(format!("error: {}", e));
            })?;
        }
        if let Some(addr) = self.ipv6.as_ref() {
            addr.parse::<Ipv4Addr>().map_err(|e| {
                return Error::Argument(format!("error: {}", e));
            })?;
        }

        if self.ipv4.is_none() && self.ipv6.is_none() {
            return Err(Error::Argument(format!(
                "No valid IPv4 or IPv6 address was specified."
            )));
        }

        Ok(true)
    }

    pub fn build(&mut self) -> Result<Box<dyn Config>, Error> {
        match self.check_valid() {
            Ok(_) => {}
            Err(e) => return Err(e),
        }

        if let Some(addr) = self.ipv4.as_ref() {
            self.addr4 = Some(SocketAddr::new(
                IpAddr::V4(addr.parse::<Ipv4Addr>().unwrap()),
                self.port
            ));
        }
        if let Some(addr) = self.ipv6.as_ref() {
            self.addr6 = Some(SocketAddr::new(
                IpAddr::V6(addr.parse::<Ipv6Addr>().unwrap()),
                self.port
            ));
        }

        Ok(Box::new(DefaultConfiguration::new(self)))
    }
}

pub struct DefaultConfiguration {
    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>,

    storage_path: String,
    bootstrap_nodes: Vec<NodeInfo>,
}

impl DefaultConfiguration {
    fn new(b: &mut Builder) -> Self {
        Self {
            addr4: b.addr4.take(),
            addr6: b.addr6.take(),
            storage_path: std::mem::take(&mut b.storage_path),
            bootstrap_nodes: std::mem::take(&mut b.bootstrap_nodes),
        }
    }
}

impl Config for DefaultConfiguration {
    fn addr4(&self) -> Option<&SocketAddr> {
        self.addr4.as_ref()
    }

    fn addr6(&self) -> Option<&SocketAddr> {
        self.addr6.as_ref()
    }

    fn storage_path(&self) -> &str {
        self.storage_path.as_str()
    }

    fn bootstrap_nodes(&self) -> &[NodeInfo] {
        &self.bootstrap_nodes
    }

    fn dump(&self) {
        println!("config: {}", self);
    }
}

impl fmt::Display for DefaultConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(v) = self.addr4.as_ref() {
            write!(f, "ipv4:{},", v)?;
        }
        if let Some(v) = self.addr6.as_ref() {
            write!(f, "ipv4:{},", v)?;
        }

        write!(f, "storage:{},", &self.storage_path)?;

        write!(f, "bootstraps: [")?;
        for item in self.bootstrap_nodes.iter() {
            write!(f, "{}, ", item)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
