use std::fmt;
use std::net::SocketAddr;
use std::marker::PhantomData;

use crate::id::Id;
use crate::value::Value;
use super::message::{
    Message,
    MessageBuidler,
    Kind,
    Method
};

pub(crate) trait StoreOption {
    fn token(&self) -> i32;
    fn value(&self) -> &Box<Value>;
}

pub(crate) trait StoreOptionBuilder {
    fn with_token(&mut self, _: i32) -> &mut Self;
    fn with_value(&mut self, _: Box<Value>) -> &mut Self;
}

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

impl StoreOption for Request {
    fn token(&self) -> i32 {
        self.token
    }

    fn value(&self) -> &Box<Value> {
        &self.value
    }
}

impl<'a,'b> MessageBuidler<'b> for RequestBuidler<'a,'b> {
    fn with_id(&mut self, id: &'b Id) -> &mut Self {
        self.id = Some(id); self
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

impl<'a,'b> StoreOptionBuilder for RequestBuidler<'a,'b> {
    fn with_token(&mut self, token: i32) -> &mut Self {
        self.token = token; self

    }
    fn with_value(&mut self, value: Box<Value>) -> &mut Self {
        self.value = Some(value); self
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

impl<'a,'b> MessageBuidler<'b> for ResponseBuilder<'a,'b> {
    fn with_id(&mut self, id: &'b Id) -> &mut Self {
        self.id = Some(id);
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

#[allow(dead_code)]
pub(crate) struct Request {
    id: Id,
    addr: SocketAddr,
    txid: i32,
    ver: i32,

    token: i32,
    value: Box<Value>,
}

pub(crate) struct RequestBuidler<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,
    txid: i32,
    ver: i32,

    token: i32,
    value: Option<Box<Value>>,

    marker: PhantomData<&'a ()>,
}

#[allow(dead_code)]
pub(crate) struct Response {
    id: Id,
    addr: SocketAddr,
    txid: i32,
    ver: i32
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
    pub(crate) fn new(b: &mut RequestBuidler) -> Self {
        Request {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver,
            token: b.token,
            value: b.value.take().unwrap(),
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
            token: 0,
            value: None,
            marker: PhantomData
        }
    }

    #[inline]
    fn is_valid(&self) -> bool {
        self.id.is_some() && self.addr.is_some()
    }

    pub(crate) fn build(&mut self) -> Request {
        assert!(self.is_valid(), "Imcomplete announce_peer request buidler");
        Request::new(self)
    }
}

#[allow(dead_code)]
impl fmt::Display for Request {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}

#[allow(dead_code)]
impl Response {
    pub(crate) fn new(b: &mut ResponseBuilder) -> Self {
        Response {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
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
            marker: PhantomData
        }
    }

    #[inline]
    fn is_valid(&self) -> bool {
        self.id.is_some() && self.addr.is_some()
    }

    pub(crate) fn build(&mut self) -> Response {
        assert!(self.is_valid(), "Imcomplete announce_peer response buidler");
        Response::new(self)
    }
}