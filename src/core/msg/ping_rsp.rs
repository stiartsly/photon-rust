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
    msg::{
        Kind, Method, Msg,
        Data as MsgData
    }
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

    fn from_cbor(&mut self, input: &ciborium::value::Value) -> bool {
        let root = match input.as_map() {
            Some(root) => root,
            None => return false,
        };

        for (k,v) in root {
            let k = match k.as_text() {
                Some(k) => k,
                None => return false,
            };
            match k {
                "y" => {},
                "t" => {
                    let v = match v.as_integer() {
                        Some(v) => v,
                        None => return false,
                    };
                    let txid = v.try_into().unwrap();
                    self.set_txid(txid);
                },
                "v" => {
                    let v = match v.as_integer() {
                        Some(v) => v,
                        None => return false,
                    };
                    let ver = v.try_into().unwrap();
                    self.set_ver(ver);
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
                Kind::Response, Method::Ping, txid
            )
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for ping_rsp message"))),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},v:{}",
            self.kind(),
            self.method(),
            self.txid(),
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
