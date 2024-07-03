use std::fmt;
use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use ciborium::Value;

use crate::{
    version,
    error::Error,
    id::Id
};

pub(crate) trait Reachable {
    fn reachable(&self) -> bool;
    fn unreachable(&self) -> bool;
    fn set_reachable(&mut self, _: bool);
}

pub(crate) trait Convertible {
    fn node(&self) -> &NodeInfo;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NodeInfo {
    id: Id,
    addr: SocketAddr,
    ver: i32,
}

impl NodeInfo {
    pub fn new(id: &Id, addr: &SocketAddr) -> Self {
        NodeInfo {
            id: id.clone(),
            addr: addr.clone(),
            ver: 0,
        }
    }

    pub(crate) fn try_from_cbor(input: &Value) -> Result<Self, Error> {
        let array = match input.as_array() {
            Some(array) => array,
            None => return Err(Error::Protocol(
                format!("Invalid cobor value for node info")))
        };

        let id = match Id::try_from_cbor(&array[0]) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let port = match array[2].as_integer() {
            Some(v) => v.try_into().unwrap(),
            None => return Err(Error::Protocol(
                format!("Port missing, invalid cobor value"))),
        };

        let addr = match array[1].as_bytes() {
            Some(addr) => addr,
            None => return Err(Error::Protocol(
                format!("Socket address missing, invalid cobor value"))),
        };

        let addr = match addr.len() {
            4 => {
                let ip: [u8;4] = addr.as_slice().try_into().unwrap();
                SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
            },
            10 => {
                let ip: [u8;16] = addr.as_slice().try_into().unwrap();
                SocketAddr::new(IpAddr::V6(Ipv6Addr::from(ip)), port)
            },
            _ => return Err(Error::Protocol(
                format!("Parsing socket adddress failed, invalid cobor value"))),
        };
        Ok(Self { id, addr, ver: 0 })
    }

    pub const fn ip(&self) -> IpAddr {
        self.addr.ip()
    }

    pub const fn port(&self) -> u16 {
        self.addr.port()
    }

    pub const fn id(&self) -> &Id {
        &self.id
    }

    pub const fn socket_addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub const fn version(&self) -> i32 {
        self.ver
    }

    pub fn set_version(&mut self, version: i32) {
        self.ver = version
    }

    pub fn is_ipv4(&self) -> bool {
        match self.addr.ip() {
            IpAddr::V4(_) => true,
            _ => false,
        }
    }

    pub fn is_ipv6(&self) -> bool {
        match self.addr.ip() {
            IpAddr::V6(_) => true,
            _ => false,
        }
    }

    pub fn formatted_version(&self) -> String {
        version::formatted_version(self.ver)
    }

    pub fn matches(&self, other: &NodeInfo) -> bool {
        self.id == other.id || self.addr == other.addr
    }

    pub(crate) fn to_cbor(&self) -> Value {
        let addr = match self.addr.ip() {
            IpAddr::V4(addr4) => Value::Bytes(addr4.octets().to_vec()),
            IpAddr::V6(addr6) => Value::Bytes(addr6.octets().to_vec()),
        };

        Value::Array(vec![
            self.id.to_cbor(),
            addr,
            Value::Integer(self.addr.port().into())
        ])
    }
}

impl Reachable for NodeInfo {
    fn reachable(&self) -> bool { false }
    fn unreachable(&self) -> bool { false }
    fn set_reachable(&mut self, _: bool) {}
}

impl Convertible for NodeInfo {
    fn node(&self) -> &NodeInfo {
        self
    }
}

impl fmt::Display for NodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{},{},{}",
            self.id,
            self.addr,
            version::formatted_version(self.ver)
        )?;
        Ok(())
    }
}
