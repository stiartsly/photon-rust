use std::any::Any;
use std::fmt;
use std::net::SocketAddr;

use super::lookup;
use super::msg::{Kind, Method, Msg};
use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::version;

pub(crate) trait ValueOption {
    fn seq(&self) -> i32;
    fn with_seq(&mut self, _: i32);
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

    fn ser(&self) -> Vec<u8> {
        unimplemented!()
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
}

impl ValueOption for Message {
    fn seq(&self) -> i32 {
        self.seq
    }

    fn with_seq(&mut self, seq: i32) {
        self.seq = seq
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,

    target: Option<Id>,
    want4: bool,
    want6: bool,
    want_token: bool,

    seq: i32,
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            target: None,
            want4: false,
            want6: false,
            want_token: false,
            seq: -1,
        }
    }

    fn want(&self) -> i32 {
        let mut want = 0;

        if self.want4 {
            want |= 0x01
        }
        if self.want6 {
            want |= 0x02
        }
        if self.want_token {
            want |= 0x04
        }

        want
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "y:{},m:{},t:{},q:{{t:{},w:{}}}",
            self.kind(),
            self.method(),
            self.txid,
            self.target.as_ref().unwrap(),
            self.want()
        )?;
        if self.seq >= 0 {
            write!(f, ",seq:{}", self.seq)?;
        }

        write!(f, ",v:{}", version::formatted_version(self.ver))?;
        Ok(())
    }
}
