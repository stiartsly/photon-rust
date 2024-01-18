use std::fmt;
use std::net::SocketAddr;
use std::marker::PhantomData;

use crate::id::Id;
use crate::version;
use super::message::{
    Message,
    MessageBuidler,
    Kind,
    Method
};

pub(crate) trait ErrorResult {
    fn msg(&self) -> &str;
    fn code(&self) -> i32;
}

pub(crate) trait ErrorResultBuilder<'a> {
    fn with_msg(&mut self, _: &'a str) -> &mut Self;
    fn with_code(&mut self, _: i32) -> &mut Self;
}

#[allow(dead_code)]
pub(crate) struct ErrorMsg {
    id: Id,
    addr: SocketAddr,

    txid: i32,
    ver: i32,

    msg: String,
    code: i32
}

#[allow(dead_code)]
pub(crate) struct ErrorMsgBuilder<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,

    txid: i32,
    ver: i32,

    code: i32,
    msg: Option<&'b str>,

    marker: PhantomData<&'a ()>,
}

impl Message for ErrorMsg {
    fn kind(&self) -> Kind {
        Kind::Error
    }

    fn method(&self) -> Method {
        Method::Unknown
    }

    fn id(&self) -> &Id {
        &self.id
    }

    fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    fn txid(&self) -> i32 {
        self.txid
    }

    fn version(&self) -> i32 {
        self.ver
    }
}

impl ErrorResult for ErrorMsg {
    fn msg(&self) -> &str {
        &self.msg
    }

    fn code(&self) -> i32 {
        self.code
    }
}

impl<'a,'b> MessageBuidler<'b> for ErrorMsgBuilder<'a,'b> {
    fn with_id(&mut self, nodeid: &'b Id) -> &mut Self {
        self.id = Some(nodeid);
        self
    }

    fn with_addr(&mut self, addr: &'b SocketAddr) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    fn with_txid(&mut self, txid: i32) -> &mut Self {
        self.txid = txid;
        self
    }

    fn with_verion(&mut self, ver: i32) -> &mut Self {
        self.ver = ver;
        self
    }
}

impl<'a,'b> ErrorResultBuilder<'b> for ErrorMsgBuilder<'a, 'b> {
    fn with_msg(&mut self, msg: &'b str) -> &mut Self {
        self.msg = Some(msg); self
    }
    fn with_code(&mut self, code: i32) -> &mut Self {
        self.code = code; self
    }
}

#[allow(dead_code)]
impl ErrorMsg {
    pub(crate) fn new(b: &ErrorMsgBuilder) -> Self {
        ErrorMsg {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver,
            code: b.code,
            msg: b.msg.unwrap().to_string()
        }
    }
}

impl fmt::Display for ErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "y:{},m:{},t:{},e:{{c:{}.m:{}}}v:{}",
            self.kind(),
            self.method(),
            self.txid,
            self.code,
            self.msg,
            version::readable_version(self.ver)
        )?;
        Ok(())
    }
}

#[allow(dead_code)]
impl<'a,'b> ErrorMsgBuilder<'a,'b> {
    pub(crate) fn new() -> Self {
        ErrorMsgBuilder {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            code: 0,
            msg: None,
            marker: PhantomData,
        }
    }
    fn is_valid(&self) -> bool {
        self.id.is_some() && self.addr.is_some() &&
            self.msg.is_some()
    }

    pub(crate) fn build(&self) -> ErrorMsg {
        assert!(self.is_valid(), "Imcomplete error msg");
        ErrorMsg::new(self)
    }
}
