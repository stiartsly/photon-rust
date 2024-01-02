use std::net::SocketAddr;
use std::marker::PhantomData;

use crate::id::Id;
use super::message::{
    Message,
    MessageBuidler,
    MsgType,
    MsgMethod
};

impl Message for Request {
    fn mtype(&self) -> MsgType {
        return MsgType::Request;
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

impl<'a,'b> MessageBuidler<'b> for RequestBuidler<'a,'b> {
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

    fn is_valid(&self) -> bool {
        false
    }
}

impl Message for Response {
    fn mtype(&self) -> MsgType {
        return MsgType::Request;
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

impl<'a, 'b> MessageBuidler<'b> for ResponseBuilder<'a,'b> {
    fn with_id(&mut self, _: &Id) -> &mut Self {
        unimplemented!()
    }

    fn with_addr(&mut self, _: &SocketAddr) -> &mut Self {
        unimplemented!()
    }

    fn with_txid(&mut self, _: i32) -> &mut Self {
        unimplemented!()
    }

    fn with_verion(&mut self, _: i32) -> &mut Self {
        unimplemented!()
    }

    fn is_valid(&self) -> bool {
        false
    }
}

#[allow(dead_code)]
pub(crate) struct Request {
    id: Id,
    addr: SocketAddr,

    txid: i32,
    ver: i32
}

pub(crate) struct Response {
    id: Id,
    addr: SocketAddr,

    txid: i32,
    ver: i32
}

pub(crate) struct RequestBuidler<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,

    txid: i32,
    ver: i32,

    marker: PhantomData<&'a ()>,
}

pub(crate) struct ResponseBuilder<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,

    txid: i32,
    ver: i32,

    marker: PhantomData<&'a ()>,
}

#[allow(dead_code)]
impl Request {
    pub(crate) fn new<'a>(b: &'a RequestBuidler) -> Self {
        Request {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver,
        }
    }
}

#[allow(dead_code)]
impl<'a,'b> RequestBuidler<'a,'b> {
    pub(crate) fn new() -> Self {
        RequestBuidler {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            marker: PhantomData,
        }
    }

    pub(crate) fn build(&self) -> Request {
        assert!(self.is_valid(), "Imcomplete request buidler");
        Request::new(self)
    }
}

impl ToString for Request {
    fn to_string(&self) -> String {
        unimplemented!()
    }
}

impl Response {
    pub(crate) fn new(b: &ResponseBuilder) -> Self {
        Response {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver,
        }
    }
}

impl ToString for Response {
    fn to_string(&self) -> String {
        unimplemented!()
    }
}

impl<'a,'b> ResponseBuilder<'a,'b> {
    pub(crate) fn new() -> Self {
        ResponseBuilder {
            id: None,
            addr: None,
            txid: 0,
            ver: 0,
            marker: PhantomData,
        }
    }

    pub(crate) fn build(&self) -> Response {
        assert!(self.is_valid(), "Imcomplete response buidler");
        Response::new(self)
    }
}
