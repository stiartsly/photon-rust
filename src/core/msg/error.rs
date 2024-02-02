use std::fmt;
use std::any::Any;
use std::net::SocketAddr;

use crate::id::Id;
use crate::version;
use super::msg::{Msg, Kind, Method};

pub(crate) trait ErrorResult {
    fn msg(&self) -> &str;
    fn code(&self) -> i32;

    fn with_msg(&mut self, _: &str);
    fn with_code(&mut self, _: i32);
}

#[allow(dead_code)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,

    msg: Option<String>,
    code: i32
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

    fn txid(&self) -> i32 {
        self.txid
    }

    fn version(&self) -> i32 {
        self.ver
    }

    fn with_id(&mut self, nodeid: &Id) {
        self.id = Some(nodeid.clone())
    }

    fn with_addr(&mut self, addr: &SocketAddr) {
        self.addr = Some(addr.clone())
    }

    fn with_txid(&mut self, txid: i32) {
        self.txid = txid
    }

    fn with_verion(&mut self, ver: i32) {
        self.ver = ver
    }

    fn as_any(&self) -> &dyn Any {
        self
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

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            code: 0,
            msg: None
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "y:{},m:{},t:{},e:{{c:{}.m:{}}}v:{}",
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
