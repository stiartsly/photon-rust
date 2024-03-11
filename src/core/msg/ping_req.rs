use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use ciborium::Value as CVal;

use crate::{
    version,
    error,
    id::Id,
    rpccall::RpcCall
};

use super::{
    keys,
    msg::{self, Kind, Method, Msg}
};

impl Msg for Message {
    fn kind(&self) -> Kind {
        Kind::from(self._type)
    }

    fn method(&self) -> Method {
        Method::from(self._type)
    }

    fn id(&self) -> &Id {
        &self.id.as_ref().unwrap()
    }

    fn addr(&self) -> &SocketAddr {
        &self.addr.as_ref().unwrap()
    }

    fn txid(&self) -> i32 {
        self.txid
    }

    fn version(&self) -> i32 {
        self.ver
    }

    fn set_id(&mut self, nodeid: Id) {
        self.id = Some(nodeid)
    }

    fn set_addr(&mut self, addr: SocketAddr) {
        self.addr = Some(addr)
    }

    fn set_txid(&mut self, txid: i32) {
        self.txid = txid
    }

    fn set_ver(&mut self, ver: i32) {
        self.ver = ver
    }

    fn associated_call(&self) -> Option<Rc<RefCell<RpcCall>>> {
        match self.associated_call.as_ref() {
            Some(call) => Some(Rc::clone(call)),
            None => None
        }
    }

    fn with_associated_call(&mut self, call: Rc<RefCell<RpcCall>>) {
        self.associated_call = Some(call);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_cbor(&self) -> CVal {
        CVal::Map(vec![
            (
                CVal::Text(String::from(keys::KEY_TYPE)),
                CVal::Integer(self._type.into())
            ),
            (
                CVal::Text(String::from(keys::KEY_TXID)),
                CVal::Integer(self.txid.into())
            ),
            (
                CVal::Text(String::from(keys::KEY_VERSION)),
                CVal::Integer(self.ver.into())
            )
        ])
    }

    fn from_cbor(&mut self, input: &ciborium::value::Value) -> bool {
        let root = input.as_map().unwrap().iter();
        for (key_cbor, val_cbor) in root {
            if !key_cbor.is_text()|| !val_cbor.is_integer(){
                return false;
            }

            let key = key_cbor.as_text().unwrap();
            let val = val_cbor.as_integer().unwrap();
            match key {
                keys::KEY_TYPE => {
                    self._type = val.try_into().unwrap()
                },
                keys::KEY_TXID => {
                    self.txid = val.try_into().unwrap()
                },
                keys::KEY_VERSION => {
                    self.ver = val.try_into().unwrap()
                },
                _ => {
                    return false;
                },
            }
        }
        true
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    associated_call: Option<Rc<RefCell<RpcCall>>>,

    _type: i32,
    txid: i32,
    ver: i32,
}

impl Message {
    pub(crate) fn new() -> Self {
        Self {
            id: None,
            addr: None,
            associated_call: None,
            _type: msg::msg_type(Kind::Request, Method::Ping),
            txid: 0,
            ver: 0,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Box<dyn Msg>, error::Error> {
        let mut msg = Box::new(Self::new());
        msg.from_cbor(input);
        Ok(msg as Box<dyn Msg>)
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
            self.txid,
            version::formatted_version(self.ver)
        )?;
        Ok(())
    }
}
