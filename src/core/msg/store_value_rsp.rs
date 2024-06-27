use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    error::Error,
};

use super::{
    msg::{
        Kind,
        Method,
        Msg,
        Data as MsgData
    }
};

pub(crate) struct Message {
    base_data: MsgData,
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn from_cbor(&mut self, input: &CVal) -> bool {
        let root = match input.as_map() {
            Some(root) => root,
            None => return false,
        };

        for (key, val) in root {
            let key = match key.as_text() {
                Some(key) => key,
                None => return false,
            };
            match key {
                "y" => {},
                "t" => {
                    let txid = match val.as_integer() {
                        Some(txid) => txid,
                        None => return false,
                    };
                    self.set_txid(txid.try_into().unwrap());
                },
                "v" => {
                    let ver = match val.as_integer() {
                        Some(ver) => ver,
                        None => return false,
                    };
                    self.set_ver(ver.try_into().unwrap());
                },
                _=> return false,
            }
        }
        true
    }

    fn ser(&self) -> CVal {
        Msg::to_cbor(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(
                Kind::Request, Method::StoreValue, txid
            ),
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for store_value_req message")
            )),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}
