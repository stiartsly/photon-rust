use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::net::SocketAddr;
use std::fmt::Debug;
use ciborium::Value as CVal;

use crate::{
    version,
    error,
    id::Id,
    rpccall::RpcCall
};

use super::{
    msg::{Kind, Method, Msg}
};

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
        unimplemented!()
    }

    fn with_associated_call(&mut self, _: Rc<RefCell<RpcCall>>) {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_cbor(&self) -> CVal {
        unimplemented!()
    }

    fn from_cbor(&mut self, _: &CVal) -> bool {
        unimplemented!()
    }

    fn target(&self) -> &Id {
        &self.target.as_ref().unwrap()
    }

    fn want4(&self) -> bool {
        self.want4
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

    fn with_want_token(&mut self) {
        self.want_token = true
    }
}

#[derive(Debug)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

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
            target: None,
            want4: false,
            want6: false,
            want_token: false,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, error::Error> {
        let msg = Rc::new(RefCell::new(Self::new()));
        msg.borrow_mut().from_cbor(input);
        Ok(msg as Rc<RefCell<dyn Msg>>)
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
