use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    error,
    value::Value,
};

use super::{
    msg::{Kind, Method, Msg, Data as MsgData},
    lookup_rsp::{Msg as LookupResponse, Data as LookupData},
};

pub(crate) struct Message {
    base_data: MsgData,
    lookup_data: LookupData,

    value: Option<Box<Value>>,
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

    fn from_cbor(&mut self, _: &CVal) -> bool {
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
            base_data: MsgData::new(Kind::Response, Method::FindValue, txid),
            lookup_data: LookupData::new(),
            value: None,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, error::Error> {
        let mut msg = Self::new();
        msg.from_cbor(input);
        Ok(Rc::new(RefCell::new(msg)))
    }

    pub(crate) fn value(&self) -> &Option<Box<crate::value::Value>> {
        &self.value
    }

    pub(crate) fn populate_value(&mut self, value: Box<crate::value::Value>) {
        self.value = Some(value)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
