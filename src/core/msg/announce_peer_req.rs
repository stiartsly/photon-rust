use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    error,
    peer::Peer,
    id::Id,
};

use super::{
    msg::{Kind, Method, Msg, Data as MsgData},
};

pub(crate) struct Message {
    base_data: MsgData,

    token: i32,
    peers: Vec<Box<Peer>>,
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn to_cbor(&self) -> CVal {
        unimplemented!()
    }

    fn from_cbor(&mut self, _: &CVal)-> bool {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(Kind::Request, Method::AnnouncePeer, txid),
            token: 0,
            peers: Vec::new(),
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, error::Error> {
        let mut msg = Self::new();
        msg.from_cbor(input);
        Ok(Rc::new(RefCell::new(msg)))
    }

    pub(crate) fn token(&self) -> i32 {
        self.token
    }

    pub(crate) fn peers(&self) -> &Vec<Box<Peer>> {
        &self.peers
    }

    pub(crate) fn with_token(&mut self, token: i32) {
        self.token = token
    }

    pub(crate) fn with_peers(&mut self, peers: Vec<Box<Peer>>) {
        self.peers = peers
    }

    pub(crate) fn target(&self) -> &Id {
        unimplemented!()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}
