use std::any::Any;
use std::fmt;
use std::net::SocketAddr;
use std::fmt::Debug;
use ciborium::value::Value;

use super::msg::{Kind, Method, Msg};
use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::version;

pub(crate) trait ErrorResult {
    fn msg(&self) -> &str;
    fn code(&self) -> i32;

    fn with_msg(&mut self, _: &str);
    fn with_code(&mut self, _: i32);
}

#[derive(Debug)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,

    msg: Option<String>,
    code: i32,
}

impl Msg for Message {
    fn kind(&self) -> Kind {
        Kind::Request
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
        self.id.as_ref().unwrap()
    }

    fn remote_addr(&self) -> &SocketAddr {
        &self.addr.as_ref().unwrap()
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
        unimplemented!()
    }

    fn from_cbor(&mut self, _: &Value) -> bool {
        unimplemented!()
    }
}

impl ErrorResult for Message {
    fn msg(&self) -> &str {
        &self.msg.as_ref().unwrap()
    }

    fn code(&self) -> i32 {
        self.code
    }

    fn with_msg(&mut self, str: &str) {
        self.msg = Some(str.to_string())
    }

    fn with_code(&mut self, code: i32) {
        self.code = code
    }
}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            code: 0,
            msg: None,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "y:{},m:{},t:{},e:{{c:{}.m:{}}}v:{}",
            self.kind(),
            self.method(),
            self.txid,
            self.code,
            self.msg.as_ref().unwrap(),
            version::formatted_version(self.ver)
        )?;
        Ok(())
    }
}
