use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::net::SocketAddr;
use ciborium::Value as CVal;

use crate::{
    version,
    id::Id,
    error::Error,
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
        let query_part = CVal::Map(vec![
            (
                CVal::Text(String::from(keys::KEY_REQ_TARGET)),
                self.target.as_ref().unwrap().to_cbor()
            ),
            (
                CVal::Text(String::from(keys::KEY_REQ_WANT)),
                CVal::Integer(self.want().into())
            )
        ]);

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
                query_part,
            )
        ])
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
                keys::KEY_TYPE => {
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    self._type = val.try_into().unwrap();
                },
                keys::KEY_TXID => {
                    let txid = match val.as_integer() {
                        Some(txid) => txid,
                        None => return false,
                    };
                    self.txid = txid.try_into().unwrap();
                },
                keys::KEY_VERSION => {
                    let ver = match val.as_integer() {
                        Some(ver) => ver,
                        None => return false,
                    };
                    self.ver = ver.try_into().unwrap();
                },
                keys::KEY_REQUEST => {
                    let map = match val.as_map() {
                        Some(map) => map,
                        None => return false,
                    };
                    for (key, val) in map {
                        let key = match key.as_text() {
                            Some(key) => key,
                            None => return false,
                        };
                        match key {
                            keys::KEY_REQ_WANT => {
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                let _want: i32 = val.try_into().unwrap();
                                self.want4 = (_want & 0x01) != 0;
                                self.want6 = (_want & 0x02) != 0;
                            },
                            keys::KEY_REQ_TARGET => {
                                let id = match Id::from_cbor(val) {
                                    Ok(id) => id,
                                    Err(_) => return false,
                                };
                                self.target = Some(id)
                            },
                            _ => return false,
                        }
                    }
                },
                _ => return false,
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

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let msg = Rc::new(RefCell::new(Self::new()));
        let mut binding = msg.borrow_mut();
        match binding.from_cbor(input) {
            true => Ok(Rc::clone(&msg) as Rc<RefCell<dyn Msg>>),
            false => Err(Error::Protocol(format!("Invalid cobor value for find_node_req message"))),
        }
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
