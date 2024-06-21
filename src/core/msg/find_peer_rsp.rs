use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    error::Error,
    peer::Peer,
};

use super::{
    msg::{
        Kind,
        Method,
        Msg,
        Data as MsgData
    },
    lookup_rsp::{
        Msg as LookupResponse,
        Data as LookuResponseData
    },
};

pub(crate) struct Message {
    base_data: MsgData,
    lookup_data: LookuResponseData,

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
    fn data(&self) -> &LookuResponseData {
        &self.lookup_data
    }

    fn data_mut(&mut self) -> &mut LookuResponseData {
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
            lookup_data: LookuResponseData::new(),
            peers: None,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(format!("Invalid cobor value for find_peer_rsp message"))),
        }
    }

    pub(crate) fn has_peers(&self) -> bool {
        self.peers.as_ref().map_or(false, |peers| !peers.is_empty())
    }

    pub(crate) fn peers(&self) -> Option<&[Box<Peer>]> {
        self.peers.as_ref().map(|peers| peers.as_slice())
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
