use std::fmt;
use std::marker::PhantomData;
use std::net::SocketAddr;

use crate::id::Id;
use crate::version;
use crate::node::Node;
use crate::peer::Peer;
use super::lookup;
use super::message::{
    Message,
    MessageBuidler,
    Kind,
    Method
};

pub(crate) trait PeerResult {
    fn has_peers(&self) -> bool;
    fn peers(&self) -> &[Box<Peer>];
}

pub(crate) trait PeerResultBuilder {
    fn populate_peers<F>(&mut self, f: F) -> &mut Self
    where F: FnMut() -> Vec<Box<Peer>>;
}

impl Message for Request {
    fn kind(&self) -> Kind {
        Kind::Request
    }

    fn method(&self) -> Method {
        Method::FindPeer
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

impl lookup::Option for Request {
    fn target(&self) -> &Id {
        &self.target
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
}

impl<'a,'b> MessageBuidler<'b> for RequestBuidler<'a,'b> {
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

impl<'a,'b> lookup::OptionBuilder<'b> for RequestBuidler<'a,'b> {
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
    fn kind(&self) -> Kind {
        return Kind::Response;
    }

    fn method(&self) -> Method {
        return Method::Ping;
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

impl lookup::Result for Response {
    fn nodes4(&self) -> &[Node] {
        &self.nodes4
    }

    fn nodes6(&self) -> &[Node] {
        &self.nodes6
    }

    fn token(&self) -> i32 {
        self.token
    }
}

impl PeerResult for Response {
    fn has_peers(&self) -> bool {
        !self.peers.is_empty()
    }

    fn peers(&self) -> &[Box<Peer>] {
        self.peers.as_slice()
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

impl<'a,'b> lookup::ResultBuilder for ResponseBuilder<'a,'b> {
    fn populate_closest_nodes4<F>(&mut self, want4: bool, f: F) -> &mut Self
    where F: FnOnce() -> Vec<Node> {
        match want4 {
            true => {self.nodes4 = Some(f()); self },
            false => self
        }
    }

    fn populate_closest_nodes6<F>(&mut self, want6: bool, f: F) -> &mut Self
    where F: FnOnce() -> Vec<Node> {
        match want6 {
            true => {self.nodes6 = Some(f()); self },
            false => self
        }
    }

    fn populate_token<F>(&mut self, want_token: bool, f: F) -> &mut Self
    where F: FnOnce() -> i32 {
        match want_token {
            true => {self.token = f(); self },
            false => self
        }
    }
}

impl<'a,'b> PeerResultBuilder for ResponseBuilder<'a,'b> {
    fn populate_peers<F>(&mut self, mut f: F) -> &mut Self
    where F: FnMut() -> Vec<Box<Peer>> {
        self.peers = Some(f()); self
    }
}

#[allow(dead_code)]
pub(crate) struct Request {
    id: Id,
    addr: SocketAddr,

    txid: i32,
    ver: i32,

    target: Id,
    want4: bool,
    want6: bool,
    want_token: bool
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
    ver: i32,

    nodes4: Vec<Node>,
    nodes6: Vec<Node>,
    token: i32,

    peers: Vec<Box<Peer>>
}

#[allow(dead_code)]
pub(crate) struct ResponseBuilder<'a,'b> {
    id: Option<&'b Id>,
    addr: Option<&'b SocketAddr>,

    txid: i32,
    ver: i32,

    nodes4: Option<Vec<Node>>,
    nodes6: Option<Vec<Node>>,
    token: i32,

    peers: Option<Vec<Box<Peer>>>,

    marker: PhantomData<&'a ()>,
}

#[allow(dead_code)]
impl Request {
    pub(crate) fn new(b: &RequestBuidler) -> Self {
        Request {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver,
            target: b.target.unwrap().clone(),
            want4: b.want4,
            want6: b.want6,
            want_token: b.want_token
        }
    }

    fn want(&self) -> i32 {
        let mut want = 0;

        if self.want4 { want |= 0x01 }
        if self.want6 { want |= 0x02 }
        if self.want_token { want |= 0x04 }

        want
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

    #[inline]
    fn is_valid(&self) -> bool {
        self.id.is_some() && self.addr.is_some() &&
            self.target.is_some()
    }

    pub(crate) fn build(&self) -> Request {
        assert!(self.is_valid(), "Imcomplete find_peer request");
        Request::new(self)
    }
}

#[allow(dead_code)]
impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "y:{},m:{},t:{},q:{{t:{},w:{}}},v:{}",
            self.kind(),
            self.method(),
            self.txid,
            self.target,
            self.want(),
            version::readable_version(self.ver)
        )?;
        Ok(())
    }
}

#[allow(dead_code)]
impl Response {
    pub(crate) fn new(b: &mut ResponseBuilder) -> Self {
        Response {
            id: b.id.unwrap().clone(),
            addr: b.addr.unwrap().clone(),
            txid: b.txid,
            ver: b.ver,
            nodes4: b.nodes4.take().unwrap(),
            nodes6: b.nodes6.take().unwrap(),
            token: b.token,
            peers: b.peers.take().unwrap()
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "y:{},m:{},t:{},r: {{",
            self.kind(),
            self.method(),
            self.txid
        )?;

        let mut first = true;
        if !self.nodes4.is_empty() {
            write!(f, "n4:")?;
            for item in self.nodes4.iter() {
                if !first {
                    first = false;
                    write!(f, ",")?;
                }
                write!(f, "[{}]", item)?;
            }
        }
        first = true;
        if !self.nodes6.is_empty() {
            write!(f, "n6:")?;
            for item in self.nodes6.iter() {
                if !first {
                    first = true;
                    write!(f, ",")?;
                }
                write!(f, "[{}]", item)?;
            }
        }

        if self.token != 0 {
            write!(f, ",tok:{}", self.token)?;
        }

        first = true;
        if !self.peers.is_empty() {
            write!(f, ",p:")?;
            for item in self.peers.iter() {
                if !first {
                    first = true;
                    write!(f, ",")?;
                }
                write!(f, "[{}]", item)?;
            }
        }

        write!(f, "}},v:{}", version::readable_version(self.ver))?;
        Ok(())
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
            peers: None,
            marker: PhantomData
        }
    }

    #[inline]
    fn is_valid(&self) -> bool {
        self.id.is_some() && self.addr.is_some() &&
            self.nodes4.is_some() && self.nodes6.is_some()
    }

    pub(crate) fn build(&mut self) -> Response {
        assert!(self.is_valid(), "Imcomplete find_peer response");
        Response::new(self)
    }
}
