use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    error,
    peer::Peer,
};

use super::{
    msg::{Kind, Method, Msg, Data as MsgData},
    lookup_rsp::{Msg as LookupResponse, Data as LookupData},
};

pub(crate) struct Message {
    base_data: MsgData,
    lookup_data: LookupData,

    peers: Option<Vec<Box<Peer>>>,
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn from_cbor(&mut self, _: &CVal) -> bool {
        unimplemented!()
    }

    fn ser(&self) -> CVal {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LookupResponse for Message {
    fn data(&self) -> &LookupData {
        &self.lookup_data
    }

    fn data_mut(&mut self) -> &mut LookupData {
        &mut self.lookup_data
    }
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(Kind::Response, Method::FindPeer, txid),
            lookup_data: LookupData::new(),
            peers: None,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, error::Error> {
        let mut msg = Self::new();
        msg.from_cbor(input);
        Ok(Rc::new(RefCell::new(msg)))
    }

    pub(crate) fn has_peers(&self) -> bool {
        unimplemented!()
    }

    pub(crate) fn peers(&self) -> &[Box<Peer>] {
        unimplemented!()
    }

    pub(crate) fn populate_peers(&mut self, peers: Vec<Box<Peer>>) {
        self.peers = Some(peers);
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "y:{},m:{},t:{},r: {{",
            self.kind(),
            self.method(),
            self.txid()
        )?;

        match self.nodes4() {
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

        match self.nodes6() {
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

        if self.token() != 0 {
            write!(f, ",tok:{}", self.token())?;
        }

        match self.peers.as_ref() {
            Some(peers) => {
                let mut first = true;
                if !peers.is_empty() {
                    write!(f, ",p:")?;
                    for item in peers.iter() {
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

        write!(f, "}},v:{}", version::formatted_version(self.ver()))?;
        Ok(())
    }
}
