use std::any::Any;
use std::fmt;
use std::net::SocketAddr;
use ciborium::value::Value;

use super::lookup;
use super::msg::{self, Kind, Method, Msg};
use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::version;
use super::keys;

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

    fn remote_id(&self) -> &Id {
        unimplemented!()
    }

    fn remote_addr(&self) -> &SocketAddr {
        unimplemented!()
    }

    fn txid(&self) -> i32 {
        self.txid
    }

    fn version(&self) -> i32 {
        self.ver
    }

    fn set_id(&mut self, nodeid: &Id) {
        self.id = Some(nodeid.clone())
    }

    fn set_addr(&mut self, addr: &SocketAddr) {
        self.addr = Some(addr.clone())
    }

    fn set_remote_id(&mut self, _: &Id) {
        unimplemented!()
    }

    fn set_remote_addr(&mut self, _: &SocketAddr) {
        unimplemented!()
    }

    fn set_txid(&mut self, txid: i32) {
        self.txid = txid
    }

    fn set_ver(&mut self, ver: i32) {
        self.ver = ver
    }

    fn associated_call(&self) -> Option<Box<RpcCall>> {
        // unimplemented!()
        None
    }

    fn with_associated_call(&mut self, _: Box<RpcCall>) {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_cbor(&self) -> Value {
        Value::Map(vec![
            (
                Value::Text(String::from(keys::KEY_TYPE)),
                Value::Integer(self._type.into())
            ),
            (
                Value::Text(String::from(keys::KEY_TXID)),
                Value::Integer(self.txid.into())
            ),
            (
                Value::Text(String::from(keys::KEY_VERSION)),
                Value::Integer(self.ver.into())
            ),
            (
                Value::Text(Kind::from(self._type).to_key().to_string()),
                lookup::Filter::to_cbor(self)
            )
        ])
    }

    fn from_cbor(&mut self, input: &Value) -> bool {
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
                            lookup::Filter::from_cbor(self, val);
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
}

impl lookup::Filter for Message {
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

    fn with_target(&mut self, target: &Id) {
        self.target = Some(target.clone())
    }

    fn with_want4(&mut self) {
        self.want4 = true
    }

    fn with_want6(&mut self) {
        self.want6 = true
    }

    fn with_token(&mut self) {
        self.want_token = true
    }

    fn to_cbor(&self) -> Value {
        Value::Map(vec![
            (
                Value::Text(String::from(keys::KEY_REQ_TARGET)),
                Value::Bytes(self.target.as_ref().unwrap().as_bytes().to_vec())
            ),
            (
                Value::Text(String::from(keys::KEY_REQ_WANT)),
                Value::Integer(self.want().into())
            )
        ])
    }

    fn from_cbor(&mut self, input: &Value) -> bool {
        if let Some(item) = input.as_map() {
            for (key, val) in item {
                if !key.is_text() {
                    return false;
                }
                if let Some(_key) = key.as_text() {
                    match _key {
                        keys::KEY_REQ_WANT => {
                            if let Some(_val) = val.as_integer() {
                                let _want: i32 = _val.try_into().unwrap();
                                self.want4 = (_want & 0x01) != 0;
                                self.want6 = (_want & 0x02) != 0;
                            }
                        },
                        keys::KEY_REQ_TARGET => {
                            if let Some(_val) = val.as_bytes() {
                                self.target = Some(Id::from_bytes(_val));
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
}

#[derive(Debug)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,
    _type: i32,
    txid: i32,
    ver: i32,

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
            target: None,
            want4: false,
            want6: false,
            want_token: false,
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
