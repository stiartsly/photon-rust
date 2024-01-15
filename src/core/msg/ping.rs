use std::net::SocketAddr;
use std::marker::PhantomData;

//use ciborium::value::Integer;
//use ciborium_io::Read;

use crate::id::Id;
use super::message::{
    Message,
    MessageBuidler,
    MessageParser,
    Kind,
    Method
};

impl Message for Request {
    fn kind(&self) -> Kind {
        Kind::Request
    }

    fn method(&self) -> Method {
        Method::Ping
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
}

impl<'a> MessageParser<'a> for RequestParser<'a> {
    fn with_cbor(&mut self, cbor: &'a [u8]) -> &mut Self {
        self.cbor = Some(cbor);
        self
    }
}

impl<'a> MessageParser<'a> for ResponseParser<'a> {
    fn with_cbor(&mut self, cbor: &'a [u8]) -> &mut Self {
        self.cbor = Some(cbor);
        self
    }
}

impl Message for Response {
    fn kind(&self) -> Kind {
        Kind::Request
    }

    fn method(&self) -> Method {
        Method::Ping
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

pub(crate) struct RequestParser<'a> {
    cbor: Option<&'a [u8]>
}

pub(crate) struct ResponseParser<'a> {
    cbor: Option<&'a [u8]>
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

    pub(crate) fn from(_: &ciborium::Value) -> Self {
        unimplemented!()
    }

    #[inline]
    fn is_valid(&self) -> bool {
        self.id.is_some() && self.addr.is_some()
    }

    pub(crate) fn build(&self) -> Request {
        assert!(self.is_valid(), "Imcomplete request buidler");
        Request::new(self)
    }
}

#[allow(dead_code)]
impl <'a> RequestParser<'a> {
    pub(crate) fn new() -> Self {
        RequestParser {
            cbor: None
        }
    }

    pub(crate) fn deser(&self) -> RequestBuidler {
        unimplemented!()
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

#[allow(dead_code)]
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

    pub(crate) fn from(_: &ciborium::Value) -> Self {
        unimplemented!()
    }

    #[inline]
    fn is_valid(&self) -> bool {
        self.id.is_some() && self.addr.is_some()
    }

    pub(crate) fn build(&self) -> Response {
        assert!(self.is_valid(), "Imcomplete response buidler");
        Response::new(self)
    }
}
