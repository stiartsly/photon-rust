use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
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
        self.addr.as_ref().unwrap()
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
            ),
            (
                CVal::Text(Kind::from(self._type).to_key().to_string()),
                CVal::Map(vec![
                    (
                        CVal::Text(String::from(keys::KEY_REQ_TARGET)),
                        self.target.as_ref().unwrap().to_cbor()
                    ),
                    (
                        CVal::Text(String::from(keys::KEY_REQ_WANT)),
                        CVal::Integer(self.want().into())
                    )
                ])
            )
        ])
    }

    fn from_cbor(&mut self, input: &CVal) -> bool {
        if let Some(root) = input.as_map() {
            for (key, val) in root {
                if !key.is_text() {
                    return false;
                }
                if let Some(_key) = key.as_text() {
                    match _key {
                        keys::KEY_TYPE => {
                            if let Some(_val) = val.as_integer() {
                                self._type = _val.try_into().unwrap()
                            }
                        },
                        keys::KEY_TXID => {
                            if let Some(_val) = val.as_integer() {
                                self.txid = _val.try_into().unwrap()
                            }
                        },
                        keys::KEY_VERSION => {
                            if let Some(_val) = val.as_integer() {
                                self.ver = _val.try_into().unwrap()
                            }
                        },

                        keys::KEY_REQUEST => {
                            if let Some(item) = val.as_map() {
                                for (__key, _val) in item {
                                    if !__key.is_text() {
                                        return false;
                                    }
                                    if let Some(__key) = __key.as_text() {
                                        match __key {
                                            keys::KEY_REQ_WANT => {
                                                if let Some(__val) = _val.as_integer() {
                                                    let _want: i32 = __val.try_into().unwrap();
                                                    self.want4 = (_want & 0x01) != 0;
                                                    self.want6 = (_want & 0x02) != 0;
                                                }
                                            },
                                            keys::KEY_REQ_TARGET => {
                                                self.target = Some(Id::from_cbor(_val));
                                            },
                                            &_ => {
                                                println!("_key: {}", __key);
                                                return false;
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        &_ => {
                            println!("_key: {}", _key);
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn target(&self) -> &Id {
        &self.target.as_ref().unwrap()
    }

    fn want4(&self) -> bool {
        self.want4
    }

    fn want6(&self) -> bool {
        self.want6
    }

    fn want_token(&self) -> bool {
        self.want_token
    }

    fn with_target(&mut self, target: Id) {
        self.target = Some(target)
    }

    fn with_want4(&mut self, want: bool) {
        self.want4 = want
    }

    fn with_want6(&mut self, want: bool) {
        self.want6 = want
    }

    fn with_want_token(&mut self) {
        self.want_token = true
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,
    _type: i32,
    txid: i32,
    ver: i32,

    associated_call: Option<Rc<RefCell<RpcCall>>>,

    target: Option<Id>,
    want4: bool,
    want6: bool,
    want_token: bool,
}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            _type: msg::msg_type(Kind::Request, Method::FindNode),
            associated_call: None,
            target: None,
            want4: false,
            want6: false,
            want_token: false,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Box<dyn Msg>, error::Error> {
        let mut msg = Box::new(Self::new());
        msg.from_cbor(input);
        Ok(msg as Box<dyn Msg>)
    }

    fn want(&self) -> i32 {
        let mut want = 0;
        if self.want4 {
            want |= 0x01;
        }
        if self.want6 {
            want |= 0x02;
        }
        if self.want_token {
            want |= 0x04;
        }
        want
    }

}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "y:{},m:{},t:{},q:{{t:{},w:{}}},v:{}",
            self.kind(),
            self.method(),
            self.txid,
            self.target.as_ref().unwrap(),
            self.want(),
            version::formatted_version(self.ver)
        )?;
        Ok(())
    }
}
