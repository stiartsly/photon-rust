use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use ciborium::Value as CVal;

use crate::{
    version,
    error::Error,
};

use super::{
    keys,
    msg::{Kind, Method, Msg, Data as MsgData}
};

pub(crate) struct Message {
    base_data: MsgData
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn to_cbor(&self) -> CVal {
        CVal::Map(vec![
            (
                CVal::Text(String::from(keys::KEY_TYPE)),
                CVal::Integer(self._type().into())
            ),
            (
                CVal::Text(String::from(keys::KEY_TXID)),
                CVal::Integer(self.txid().into())
            ),
            (
                CVal::Text(String::from(keys::KEY_VERSION)),
                CVal::Integer(self.ver().into())
            )
        ])
    }

    fn from_cbor(&mut self, input: &ciborium::value::Value) -> bool {
        let root = match input.as_map() {
            Some(root) => root,
            None => return false,
        };

        for (key, val) in root {
            let key = match key.as_text() {
                Some(key) => key,
                None => return false,
            };
            let val = match val.as_integer() {
                Some(val) => val,
                None => return false,
            };

            match key {
                keys::KEY_TYPE =>
                    self.set_type(Kind::Request, Method::Ping),
                keys::KEY_TXID =>
                    self.set_txid(val.try_into().unwrap()),
                keys::KEY_VERSION =>
                    self.set_ver(val.try_into().unwrap()),
                _ => return false,
            }
        }
        true
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
            base_data: MsgData::new(Kind::Request, Method::Ping, txid)
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for ping_req message"))),
        }
    }
}

#[allow(dead_code)]
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "y:{},m:{},t:{},v:{}",
            self.kind(),
            self.method(),
            self.txid(),
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
