use std::any::Any;
use std::fmt;
use std::net::SocketAddr;
use std::fmt::Debug;
use ciborium;

use super::msg::{Kind, Method, Msg};
use crate::id::Id;
use crate::node_info::NodeInfo;
use crate::rpccall::RpcCall;
use crate::value::Value;

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

    fn to_cbor(&self) -> ciborium::value::Value {
        unimplemented!()
    }

    fn from_cbor(&mut self, _: &ciborium::value::Value) -> bool {
        unimplemented!()
    }

    fn nodes4(&self) -> &[NodeInfo] {
        &self.nodes4.as_ref().unwrap()
    }

    fn nodes6(&self) -> &[NodeInfo] {
        &self.nodes6.as_ref().unwrap()
    }

    fn token(&self) -> i32 {
        self.token
    }

    fn populate_closest_nodes4(&mut self, f: Box<dyn FnOnce() -> Vec<NodeInfo>>) {
        self.nodes4 = Some(f())
    }

    fn populate_closest_nodes6(&mut self, f: Box<dyn FnOnce() -> Vec<NodeInfo>>) {
        self.nodes6 = Some(f())
    }

    fn populate_token(&mut self, want_token: bool, f: Box<dyn FnOnce() -> i32>)
    {
        match want_token {
            true => self.token = f(),
            false => {}
        }
    }

    fn value(&self) -> &Option<Box<crate::value::Value>> {
        &self.value
    }

    fn populate_value(&mut self, mut f: Box<dyn FnMut() -> Option<Box<crate::value::Value>>>)
    {
        self.value = f()
    }
}

#[derive(Debug)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,

    nodes4: Option<Vec<NodeInfo>>,
    nodes6: Option<Vec<NodeInfo>>,
    token: i32,

    value: Option<Box<Value>>,
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
            value: None,
        }
    }

    pub(crate) fn from(input: &ciborium::value::Value ) -> Self {
        let mut msg = Self::new();
        msg.from_cbor(input);
        msg
    }
}

impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
