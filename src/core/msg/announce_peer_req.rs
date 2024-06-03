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
    peer::Peer,
    rpccall::RpcCall
};

use super::{
    msg::{Kind, Method, Msg}
};

pub(crate) trait AnnounceOption {
    fn token(&self) -> i32;
    fn peers(&self) -> &Vec<Box<Peer>>;

    fn with_token(&mut self, _: i32);
    fn with_peers(&mut self, _: Vec<Box<Peer>>);
}

impl Msg for Message {
    fn kind(&self) -> Kind {
        Kind::Request
    }

    fn method(&self) -> Method {
        Method::Ping
    }

    fn id(&self) -> &Id {
        self.id.as_ref().unwrap()
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

    fn from_cbor(&mut self, _: &CVal)-> bool {
        unimplemented!()
    }
}

impl AnnounceOption for Message {
    fn token(&self) -> i32 {
        self.token
    }

    fn peers(&self) -> &Vec<Box<Peer>> {
        &self.peers
    }

    fn with_token(&mut self, token: i32) {
        self.token = token
    }

    fn with_peers(&mut self, peers: Vec<Box<Peer>>) {
        self.peers = peers
    }
}

#[derive(Debug)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,
    txid: i32,
    ver: i32,

    token: i32,
    peers: Vec<Box<Peer>>,
}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            token: 0,
            peers: Vec::new(),
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
        unimplemented!();
    }
}
