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
    msg::{Kind, Method, Msg, Data as MsgData}
};

pub(crate) struct Message {
    base_data: MsgData,

    token: i32,
    value: Option<Box<Value>>,
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn from_cbor(&mut self, _: &ciborium::value::Value) -> bool {
        unimplemented!()
    }

    fn ser(&self) -> CVal {
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
            base_data: MsgData::new(Kind::Request, Method::StoreValue, txid),
            token: 0,
            value: None,
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

    pub(crate) fn value(&self) -> &Option<Box<Value>> {
        &self.value
    }

    //fn with_token(&mut self, token: i32) {
    //    self.token = token
    //}

    pub(crate) fn with_value(&mut self, value: Box<Value>) {
        self.value = Some(value)
    }
}

#[allow(dead_code)]
impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}
