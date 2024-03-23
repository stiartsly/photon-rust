use std::any::Any;
use std::fmt;
use std::net::SocketAddr;
use ciborium::value::Value;

use super::msg::{self, Kind, Method, Msg};
use super::keys;
use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::version;

impl Msg for Message {
    fn kind(&self) -> Kind {
        Kind::Response
    }

    fn method(&self) -> Method {
        Method::Ping
    }

    fn id(&self) -> &Id {
        &self.id.as_ref().unwrap()
    }

    fn addr(&self) -> &SocketAddr {
        &self.addr.as_ref().unwrap()
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
        unimplemented!()
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
            )
        ])
    }

    fn from_cbor(&mut self, _: Value) {
        unimplemented!()
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    _type: i32,
    txid: i32,
    ver: i32,
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            _type: msg::msg_type(Kind::Response, Method::Ping),
            txid: 0,
            ver: 0,
        }
    }
}

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
