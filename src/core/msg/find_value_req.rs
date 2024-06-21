use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    error::Error,
};

use super::{
    msg::{
        Kind,
        Method,
        Msg,
        Data as MsgData
    },
    lookup_req::{
        Msg as LookupRequest,
        Data as LookupRequestData
    },
};

pub(crate) struct Message {
    base_data: MsgData,
    lookkup_data: LookupRequestData,

    seq: i32,
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

impl LookupRequest for Message {
    fn data(&self) -> &LookupRequestData {
        &self.lookkup_data
    }

    fn data_mut(&mut self) -> &mut LookupRequestData {
        &mut self.lookkup_data
    }
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(Kind::Request, Method::FindValue, txid),
            lookkup_data: LookupRequestData::new(),
            seq: -1
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for find_value_req message")
            )),
        }
    }

    pub(crate) fn seq(&self) -> i32 {
        self.seq
    }

    pub(crate) fn with_seq(&mut self, seq: i32) {
        self.seq = seq
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},q:{{t:{},w:{}}}",
            self.kind(),
            self.method(),
            self.txid(),
            self.target(),
            self.want()
        )?;
        if self.seq >= 0 {
            write!(f, ",seq:{}", self.seq)?;
        }

        write!(f, ",v:{}", version::formatted_version(self.ver()))?;
        Ok(())
    }
}
