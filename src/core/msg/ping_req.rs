use std::any::Any;
use std::fmt;
use std::net::SocketAddr;

//use ciborium::value::Integer;
//use ciborium_io::Read;

use super::msg::{Kind, Method, Msg};
use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::version;

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

    fn with_ver(&mut self, ver: i32) {
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

    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }
}

#[allow(dead_code)]
pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    txid: i32,
    ver: i32,
}

impl Message {
    pub(crate) fn new<'a>() -> Self {
        Message {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
        }
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
