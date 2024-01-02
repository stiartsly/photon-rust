use std::marker::PhantomData;
use std::net::SocketAddr;

use crate::id::Id;
use crate::nodeinfo::NodeInfo;
use super::message::{
    Message,
    MessageBuidler,
    MsgType,
    MsgMethod
};
use super::lookup::{
    Lookup,
    LookupBuilder,
    LookupResult,
    LookupResultBuilder
};

impl Message for Request {
    fn mtype(&self) -> MsgType {
        return MsgType::Request;
    }

    fn method(&self) -> MsgMethod {
        return MsgMethod::Ping;
    }

    fn id(&self) -> &Id {
        unimplemented!()
    }

    fn addr(&self) -> &SocketAddr {
        unimplemented!()
    }

    fn txid(&self) -> i32 {
        unimplemented!()
    }

    fn version(&self) -> i32 {
        unimplemented!()
    }
}

impl Lookup for Request {
    fn target(&self) -> &Id {
        unimplemented!()
    }

    fn want4(&self) -> bool {
        unimplemented!()
    }

    fn want6(&self) -> bool {
        unimplemented!()
    }

    fn want_token(&self) -> bool {
        unimplemented!()
    }
}

impl<'a,'b> MessageBuidler<'b> for RequestBuidler<'a,'b> {
    fn with_id(&mut self, id: &'b Id) -> &mut Self {
        self.id = Some(id);
        self
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

impl<'a,'b> LookupBuilder<'b> for RequestBuidler<'a,'b> {
    fn with_target(&mut self, target: &'b Id) -> &mut Self {
        self.target = Some(target);
        self
    }

    fn with_want4(&mut self) -> &mut Self {
        self.want4 = true;
        self
    }

    fn with_want6(&mut self) -> &mut Self {
        self.want6 = true;
        self
    }

    fn with_token(&mut self) -> &mut Self {
        self.want_token = true;
        self
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

impl LookupResult for Response {
    fn nodes4(&self) -> &[NodeInfo] {
        unimplemented!()
    }

    fn nodes6(&self) -> &[NodeInfo] {
        unimplemented!()
    }

    fn token(&self) -> i32 {
        unimplemented!()
    }
}

impl<'a,'b> MessageBuidler<'b> for ResponseBuilder<'a,'b> {
    fn with_id(&mut self, _: &'b Id) -> &mut Self {
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

impl<'a,'b> LookupResultBuilder for ResponseBuilder<'a,'b> {
    fn populate_closest_nodes4<F>(&mut self, want4: bool, f: F) -> &mut Self
    where F: Fn() -> Vec<NodeInfo> {
        match want4 {
            true => {self.nodes4 = Some(f()); self },
            false => self
        }
    }

    fn populate_closest_nodes6<F>(&mut self, want6: bool, f: F) -> &mut Self
    where F: Fn() -> Vec<NodeInfo> {
        match want6 {
            true => {self.nodes6 = Some(f()); self },
            false => self
        }
    }

    fn populate_token<F>(&mut self, want_token: bool, f: F) -> &mut Self
    where F: Fn() -> i32 {
        match want_token {
            true => {self.token = f(); self },
            false => self
        }
    }
}

#[allow(dead_code)]
pub(crate) struct Request {
    id: Id,
    addr: SocketAddr,

    txid: i32,
    ver: i32
}

#[allow(dead_code)]
pub(crate) struct RequestBuidler<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,

    txid: i32,
    ver: i32,

    target: Option<&'b Id>,

    want4: bool,
    want6: bool,
    want_token: bool,

    marker: PhantomData<&'a ()>,
}

#[allow(dead_code)]
pub(crate) struct Response {
    id: Id,
    addr: SocketAddr,

    txid: i32,
    ver: i32
}

#[allow(dead_code)]
pub(crate) struct ResponseBuilder<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,

    txid: i32,
    ver: i32,

    nodes4: Option<Vec<NodeInfo>>,
    nodes6: Option<Vec<NodeInfo>>,
    token: i32,

    marker: PhantomData<&'a ()>,
}

#[allow(dead_code)]
impl Request {
    pub(crate) fn new(b: &RequestBuidler) -> Self {
        Request {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver
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
            target: None,
            want4: false,
            want6: false,
            want_token: false,
            marker: PhantomData
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

#[allow(dead_code)]
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
            nodes4: None,
            nodes6: None,
            token: 0,
            marker: PhantomData
        }
    }

    pub(crate) fn build(&self) -> Response {
        assert!(self.is_valid(), "Imcomplete response buidler");
        Response::new(self)
    }
}
