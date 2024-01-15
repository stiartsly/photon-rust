use std::net::SocketAddr;
use std::marker::PhantomData;

use crate::id::Id;
use super::message::{
    Message,
    MessageBuidler,
    MsgKind,
    MsgMethod
};

#[allow(dead_code)]
pub(crate) struct ErrorMsg {
    id: Id,
    addr: SocketAddr,

    remote_id: Id,
    remote_addr: SocketAddr,

    txid: i32,
    ver: i32
}

#[allow(dead_code)]
pub(crate) struct ErrorMsgBuilder<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,

    remote_id: Option<&'b Id>,
    remote_addr: Option<&'b SocketAddr>,

    txid: i32,
    ver: i32,

    marker: PhantomData<&'a ()>,
}

impl Message for ErrorMsg {
    fn kind(&self) -> MsgKind {
        return MsgKind::Request;
    }

    fn method(&self) -> MsgMethod {
        return MsgMethod::Ping;
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

impl<'a,'b> MessageBuidler<'b> for ErrorMsgBuilder<'a,'b> {
    fn with_id(&mut self, nodeid: &'b Id) -> &mut Self {
        self.id = Some(nodeid);
        self
    }

    fn with_addr(&mut self, addr: &'b SocketAddr) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    fn with_txid(&mut self, _: i32) -> &mut Self {
        unimplemented!()
    }

    fn with_verion(&mut self, _: i32) -> &mut Self {
        unimplemented!()
    }
}

impl ToString for ErrorMsg {
    fn to_string(&self) -> String {
        // TODO:
        String::new()
    }
}
