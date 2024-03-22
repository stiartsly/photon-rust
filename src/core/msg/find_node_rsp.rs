use std::any::Any;
use std::fmt;
use std::net::SocketAddr;

use super::lookup;
use super::msg::{self, Kind, Method, Msg};
use crate::id::Id;
use crate::node_info::NodeInfo;
use crate::rpccall::RpcCall;
use crate::version;

impl Msg for Message {
    fn kind(&self) -> Kind {
        Kind::Response
    }

    fn method(&self) -> Method {
        Method::Ping
    }

    fn id(&self) -> &Id {
        &self.id.as_ref().unwrap()
    }

    fn addr(&self) -> &SocketAddr {
        &self.addr.as_ref().unwrap()
    }

    fn remote_id(&self) -> &Id {
        unimplemented!()
    }

    fn remote_addr(&self) -> &SocketAddr {
        unimplemented!()
    }

    fn txid(&self) -> i32 {
        self.txid
    }

    fn version(&self) -> i32 {
        self.ver
    }

    fn set_id(&mut self, nodeid: &Id) {
        self.id = Some(nodeid.clone())
    }

    fn set_addr(&mut self, addr: &SocketAddr) {
        self.addr = Some(addr.clone())
    }

    fn set_remote_id(&mut self, _: &Id) {
        unimplemented!()
    }

    fn set_remote_addr(&mut self, _: &SocketAddr) {
        unimplemented!()
    }

    fn set_txid(&mut self, txid: i32) {
        self.txid = txid
    }

    fn set_ver(&mut self, ver: i32) {
        self.ver = ver
    }

    fn associated_call(&self) -> Option<Box<RpcCall>> {
        unimplemented!()
    }

    fn with_associated_call(&mut self, _: Box<RpcCall>) {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn ser(&self) -> Vec<u8> {
        unimplemented!()
    }
}

impl lookup::Result for Message {
    fn nodes4(&self) -> &[NodeInfo] {
        &self.nodes4.as_ref().unwrap()
    }

    fn nodes6(&self) -> &[NodeInfo] {
        &self.nodes6.as_ref().unwrap()
    }

    fn token(&self) -> i32 {
        self.token
    }

    fn populate_closest_nodes4<F>(&mut self, want4: bool, f: F)
    where
        F: FnOnce() -> Option<Vec<NodeInfo>>,
    {
        match want4 {
            true => self.nodes4 = f(),
            false => {}
        }
    }

    fn populate_closest_nodes6<F>(&mut self, want6: bool, f: F)
    where
        F: FnOnce() -> Option<Vec<NodeInfo>>,
    {
        match want6 {
            true => self.nodes6 = f(),
            false => {}
        }
    }

    fn populate_token<F>(&mut self, want_token: bool, f: F)
    where
        F: FnOnce() -> i32,
    {
        match want_token {
            true => self.token = f(),
            false => {}
        }
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    _type: i32,
    txid: i32,
    ver: i32,

    nodes4: Option<Vec<NodeInfo>>,
    nodes6: Option<Vec<NodeInfo>>,
    token: i32,
}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            _type: msg::msg_type(Kind::Response, Method::FindNode),
            txid: 0,
            ver: 0,
            nodes4: None,
            nodes6: None,
            token: 0,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "y:{},m:{},t:{},r: {{",
            self.kind(),
            self.method(),
            self.txid
        )?;

        match self.nodes4.as_ref() {
            Some(nodes4) => {
                let mut first = true;
                if !nodes4.is_empty() {
                    write!(f, "n4:")?;
                    for item in nodes4.iter() {
                        if !first {
                            first = false;
                            write!(f, ",")?;
                        }
                        write!(f, "[{}]", item)?;
                    }
                }
            }
            None => {}
        }

        match self.nodes6.as_ref() {
            Some(nodes6) => {
                let mut first = true;
                if !nodes6.is_empty() {
                    write!(f, "n4:")?;
                    for item in nodes6.iter() {
                        if !first {
                            first = false;
                            write!(f, ",")?;
                        }
                        write!(f, "[{}]", item)?;
                    }
                }
            }
            None => {}
        }

        if self.token != 0 {
            write!(f, ",tok:{}", self.token)?;
        }
        write!(f, ",v:{}", version::formatted_version(self.ver))?;
        Ok(())
    }
}
