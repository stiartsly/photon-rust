use std::env;
use std::fmt;
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
    input_ipv4: Option<&'a str>,
    input_ipv6: Option<&'a str>,
    port: u16,

    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>,
    storage_path: String,
    bootstrap_nodes: Vec<Node>,
}

impl DefaultConfiguration {
    fn new(b: &mut Builder) -> Self {
        DefaultConfiguration {
            addr4: b.addr4.take(),
            addr6: b.addr6.take(),
            storage_path: std::mem::take(&mut b.storage_path),
            bootstrap_nodes: std::mem::take(&mut b.bootstrap_nodes),
        }
    }
}

impl Config for DefaultConfiguration {
    fn addr4(&self) -> &Option<SocketAddr> {
        &self.addr4
    }

    fn addr6(&self) -> &Option<SocketAddr> {
        &self.addr6
    }

    fn storage_path(&self) -> &str {
        &self.storage_path
    }

    fn bootstrap_nodes(&self) -> &[Node] {
        &self.bootstrap_nodes
    }

    fn dump(&self) {
        println!("config: {}", self);
    }
}

impl fmt::Display for DefaultConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.addr4.is_some() {
            write!(f, "ipv4:{},", self.addr4.unwrap())?;
        }
        if self.addr6.is_some() {
            write!(f, "ipv6:{},", self.addr6.unwrap())?;
        }

        write!(f, "storage:{},", self.storage_path.as_str())?;
        write!(f, "bootstraps: [")?;
        for item in self.bootstrap_nodes.iter() {
            write!(f, "{}, ", item)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[allow(dead_code)]
impl<'a> Builder<'a> {
    pub fn new() -> Builder<'a> {
        let def_path = match env::var("HOME") {
            Ok(value) => value,
            Err(_) => ".".to_string()
        };

        Builder {
            input_ipv4: None,
            input_ipv6: None,
            port: 0,

            addr4: None,
            addr6: None,

            storage_path: def_path,
            bootstrap_nodes: Vec::new(),
        }
    }

    pub fn with_ipv4(&mut self, input: &'a str) -> &mut Self {
        self.input_ipv4 = Some(input); self
    }

    pub fn with_ipv6(&mut self, input: &'a str) -> &mut Self {
        self.input_ipv6 = Some(input); self
    }

    pub fn with_listening_port(&mut self, port: u16) -> &mut Self {
        self.port = port; self
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

    pub fn add_bootstrap(&mut self, node: &Node) -> &mut Self {
        self.bootstrap_nodes.push(node.clone()); self
    }

    pub fn add_bootstraps(&mut self, nodes: &[Node]) -> &mut Self {
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
            return Err(Error::Argument(format!("Invalid port value {}", self.port)));
        }

        if self.input_ipv4.is_some() {
            match self.input_ipv4.unwrap().parse::<Ipv4Addr>() {
                Ok(_) => {},
                Err(e) => {
                    return Err(Error::Argument(format!("error: {}", e)));
                }
            };
        }
        if self.input_ipv6.is_some() {
            match self.input_ipv6.unwrap().parse::<Ipv6Addr>() {
                Ok(_) => {},
                Err(e) => {
                    return Err(Error::Argument(format!("error: {}", e)));
                }
            };
        }

        if self.input_ipv4.is_none() && self.input_ipv6.is_none() {
            return Err(Error::Argument(
                format!("No valid IPv4 or IPv6 address was specified.")));
        }

        Ok(true)
    }

    pub fn build(&mut self) -> Result<Box<dyn Config>, Error> {
        match self.check_valid() {
            Ok(_) => {},
            Err(e) => { return Err(e) }
        }

        if self.input_ipv4.is_some() {
            let addr = self.input_ipv4.unwrap().parse::<Ipv4Addr>().unwrap();
            self.addr4 = Some(SocketAddr::new(IpAddr::V4(addr), self.port));
        }

        if self.input_ipv6.is_some() {
            let addr = self.input_ipv6.unwrap().parse::<Ipv6Addr>().unwrap();
            self.addr6 = Some(SocketAddr::new(IpAddr::V6(addr), self.port));
        }

        Ok(
            Box::new(DefaultConfiguration::new(self)) as Box<dyn Config>
        )
    }
}
