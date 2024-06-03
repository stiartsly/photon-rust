use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::net::SocketAddr;
use std::fmt::Debug;
use ciborium::Value as CVal;

use crate::{
    error,
    id::Id,
    node_info::NodeInfo,
    value::Value,
    rpccall::RpcCall
};

use super::{
    msg::{Kind, Method, Msg}
};

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

    fn set_id(&mut self, nodeid: Id) {
        self.id = Some(nodeid)
    }

    fn set_addr(&mut self, addr: SocketAddr) {
        self.addr = Some(addr)
    }

    fn set_txid(&mut self, txid: i32) {
        self.txid = txid
    }

    fn set_ver(&mut self, ver: i32) {
        self.ver = ver
    }

    fn associated_call(&self) -> Option<Rc<RefCell<RpcCall>>> {
        unimplemented!()
    }

    fn with_associated_call(&mut self, _: Rc<RefCell<RpcCall>>) {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_cbor(&self) -> CVal {
        unimplemented!()
    }

    fn from_cbor(&mut self, _: &CVal) -> bool {
        unimplemented!()
    }

    fn nodes4(&self) -> &[NodeInfo] {
        &self.nodes4.as_ref().unwrap()
    }

    fn token(&self) -> i32 {
        self.token
    }

    fn populate_closest_nodes4(&mut self, nodes: Vec<NodeInfo>) {
        self.nodes4 = Some(nodes)
    }

    fn populate_token(&mut self, token: i32) {
        self.token = token
    }

    fn value(&self) -> &Option<Box<crate::value::Value>> {
        &self.value
    }

    fn populate_value(&mut self, value: Box<crate::value::Value>) {
        self.value = Some(value)
    }
}

#[derive(Debug)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,

    nodes4: Option<Vec<NodeInfo>>,
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
            token: 0,
            value: None,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, error::Error> {
        let msg = Rc::new(RefCell::new(Self::new()));
        msg.borrow_mut().from_cbor(input);
        Ok(msg as Rc<RefCell<dyn Msg>>)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
