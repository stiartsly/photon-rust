use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    error,
};

use super::{
    msg::{Kind, Method, Msg, Data as MsgData}
};

pub(crate) struct Message {
    base_data: MsgData,
    msg: String,
    code: i32,
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

#[allow(dead_code)]
impl Message {
    pub(crate) fn new(method: Method, txid: i32) -> Self {
        Message {
            base_data: MsgData::new(Kind::Error, method, txid),
            code: 0,
            msg: String::from(""),
        }
    }

    pub(crate) fn from(_: &CVal) -> Result<Rc<RefCell<dyn Msg>>, error::Error> {
        /*let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(format!("Invalid cobor value for find_node request message"))),
        }*/
        unimplemented!()
    }

    pub(crate) fn msg(&self) -> &str {
        &self.msg
    }

    pub(crate) fn code(&self) -> i32 {
        self.code
    }

    pub(crate) fn with_msg(&mut self, str: &str) {
        self.msg = String::from(str);
    }

    pub(crate) fn with_code(&mut self, code: i32) {
        self.code = code
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},e:{{c:{}.m:{}}}v:{}",
            self.kind(),
            self.method(),
            self.txid(),
            self.code(),
            self.msg(),
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
