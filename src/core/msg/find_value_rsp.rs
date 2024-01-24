use std::fmt;
use std::any::Any;
use std::net::SocketAddr;

use crate::id::Id;
use crate::node::Node;
use crate::value::Value;
use super::lookup;
use super::msg::{
    Msg,
    Kind,
    Method
};

pub(crate) trait ValueResult {
    fn value(&self) -> &Option<Box<Value>>;
    fn populate_value<F>(&mut self, f: F) where F: FnMut() -> Option<Box<Value>>;
}

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

    fn as_any(&self) -> &dyn Any {
        self
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

impl ValueResult for Message {
    fn value(&self) -> &Option<Box<Value>> {
        &self.value
    }

    fn populate_value<F>(&mut self, mut f: F)
    where F: FnMut() -> Option<Box<Value>> {
        self.value = f()
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,

    nodes4: Option<Vec<Node>>,
    nodes6: Option<Vec<Node>>,
    token: i32,

    value: Option<Box<Value>>
}

#[allow(dead_code)]
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
            value: None
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}