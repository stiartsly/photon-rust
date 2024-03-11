use std::fmt;
use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use ciborium::value::Value;

use crate::id::Id;
use crate::version;

pub(crate) trait Reachable {
    fn reachable(&self) -> bool {
        false
    }
    fn unreachable(&self) -> bool {
        false
    }
    fn set_reachable(&mut self, _: bool) {}
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

    pub(crate) fn try_from_cbor(input: &Value) -> Option<Self> {
        let mut result = None;

        if let Some(array) = input.as_array().as_ref() {
            let id = Id::from_cbor(&array[0]);
            let mut port = 0;
            if let Some(_port) = array[2].as_integer() {
                port = _port.try_into().unwrap()
            }

            let mut addr = None;
            if let Some(bytes) = array[1].as_bytes() {
                if bytes.len() == 4 {
                    let ip: [u8;4] = bytes.as_slice().try_into().unwrap();
                    addr = Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port));
                } else if bytes.len() == 16 {
                    let ip: [u8;16] = bytes.as_slice().try_into().unwrap();
                    addr = Some(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(ip)), port));
                } else {
                    panic!("invalid bytes length");
                }
            }
            result = Some(
                Self { id, addr: addr.unwrap(), ver: 0}
            )
        }
        result
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

impl Reachable for NodeInfo {}

impl fmt::Display for NodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{}",
            self.id,
            self.addr,
            version::formatted_version(self.ver)
        )?;
        Ok(())
    }
}
