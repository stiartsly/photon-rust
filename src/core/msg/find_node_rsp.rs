use std::fmt;
use std::net::SocketAddr;

use crate::id::Id;
use crate::version;
use crate::node::Node;
use super::lookup;
use super::msg::{Msg, Kind, Method};

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

    fn txid(&self) -> i32 {
        self.txid
    }

    fn version(&self) -> i32 {
        self.ver
    }

    fn with_id(&mut self, nodeid: &Id) {
        self.id = Some(nodeid.clone())
    }

    fn with_addr(&mut self, addr: &SocketAddr) {
        self.addr = Some(addr.clone())
    }

    fn with_txid(&mut self, txid: i32) {
        self.txid = txid
    }

    fn with_verion(&mut self, ver: i32) {
        self.ver = ver
    }
}

impl lookup::Result for Message {
    fn nodes4(&self) -> &[Node] {
        &self.nodes4.as_ref().unwrap()
    }

    fn nodes6(&self) -> &[Node] {
        &self.nodes6.as_ref().unwrap()
    }

    fn token(&self) -> i32 {
        self.token
    }

    fn populate_closest_nodes4<F>(&mut self, want4: bool, f: F)
    where F: FnOnce() -> Option<Vec<Node>> {
        match want4 {
            true => {self.nodes4 = f()},
            false => {}
        }
    }

    fn populate_closest_nodes6<F>(&mut self, want6: bool, f: F)
    where F: FnOnce() -> Option<Vec<Node>> {
        match want6 {
            true => {self.nodes6 = f()},
            false => {}
        }
    }

    fn populate_token<F>(&mut self, want_token: bool, f: F)
    where F: FnOnce() -> i32 {
        match want_token {
            true => {self.token = f()},
            false => {}
        }
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,

    nodes4: Option<Vec<Node>>,
    nodes6: Option<Vec<Node>>,
    token: i32
}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
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
        write!(f, "y:{},m:{},t:{},r: {{",
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
                    };
                }
            },
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
                    };
                }
            },
            None => {}
        }

        if self.token != 0 {
            write!(f, ",tok:{}", self.token)?;
        }
        write!(f, ",v:{}", version::readable_version(self.ver))?;
        Ok(())
    }
}
