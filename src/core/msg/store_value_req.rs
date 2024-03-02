use std::any::Any;
use std::fmt;
use std::net::SocketAddr;

use super::msg::{Kind, Method, Msg};
use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::value::Value;

pub(crate) trait StoreOption {
    fn token(&self) -> i32;
    fn value(&self) -> &Box<Value>;

    fn with_token(&mut self, _: i32);
    fn with_value(&mut self, _: Box<Value>);
}

impl Msg for Message {
    fn kind(&self) -> Kind {
        Kind::Request
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

    fn with_ver(&mut self, ver: i32) {
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
}

impl StoreOption for Message {
    fn token(&self) -> i32 {
        self.token
    }

    fn value(&self) -> &Box<Value> {
        &self.value.as_ref().unwrap()
    }

    fn with_token(&mut self, token: i32) {
        self.token = token
    }
    fn with_value(&mut self, value: Box<Value>) {
        self.value = Some(value)
    }
}

#[allow(dead_code)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,
    txid: i32,
    ver: i32,

    token: i32,
    value: Option<Box<Value>>,
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            token: 0,
            value: None,
        }
    }
}

#[allow(dead_code)]
impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}
