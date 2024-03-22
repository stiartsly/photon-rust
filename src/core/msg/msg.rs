use std::any::Any;
use std::fmt;
use std::net::SocketAddr;

use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::error::Error;

#[derive(PartialEq)]
pub(crate) enum Kind {
    Error = 0,
    Request = 0x20,
    Response = 0x40,
}

impl Kind {
    const MASK: i32 = 0xE0;
    pub(crate) fn from(_type: i32) -> Kind {
        let kind: i32 = _type & Self::MASK;
        match kind {
            0x00 => Kind::Error,
            0x20 => Kind::Request,
            0x40 => Kind::Response,
            _ => {
                panic!("invalid msg kind: {}", kind)
            }
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Kind::Error => "e",
            Kind::Request => "q",
            Kind::Response => "r",
        };
        write!(f, "{}", str)?;
        Ok(())
    }
}

#[derive(PartialEq)]
pub(crate) enum Method {
    Unknown = 0x00,
    Ping = 0x01,
    FindNode = 0x02,
    AnnouncePeer = 0x03,
    FindPeer = 0x04,
    StoreValue = 0x05,
    FindValue = 0x6,
}

impl Method {
    const MASK: i32 = 0x1F;
    pub(crate) fn from(_type: i32) -> Self {
        let method: i32 = _type & Self::MASK;
        match method {
            0x00 => Method::Unknown,
            0x01 => Method::Ping,
            0x02 => Method::FindNode,
            0x03 => Method::AnnouncePeer,
            0x04 => Method::FindPeer,
            0x05 => Method::StoreValue,
            0x06 => Method::FindValue,
            _ => {
                panic!("invalid msg method: {}", method)
            }
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Method::Unknown => "unknown",
            Method::Ping => "ping",
            Method::FindNode => "find_node",
            Method::AnnouncePeer => "announce_peer",
            Method::FindPeer => "find_peer",
            Method::StoreValue => "store_value",
            Method::FindValue => "find_value",
        };
        write!(f, "{}", str)?;
        Ok(())
    }
}

pub(crate) trait Msg {
    fn kind(&self) -> Kind;
    fn method(&self) -> Method;

    fn id(&self) -> &Id;
    fn addr(&self) -> &SocketAddr;

    fn remote_id(&self) -> &Id;
    fn remote_addr(&self) -> &SocketAddr;

    fn txid(&self) -> i32;
    fn version(&self) -> i32;

    fn set_id(&mut self, _: &Id);
    fn set_addr(&mut self, _: &SocketAddr);

    fn set_remote_id(&mut self, _: &Id);
    fn set_remote_addr(&mut self, _: &SocketAddr);

    fn set_txid(&mut self, _: i32);
    fn set_ver(&mut self, _: i32);

    fn associated_call(&self) -> Option<Box<RpcCall>>;
    fn with_associated_call(&mut self, _: Box<RpcCall>);

    fn as_any(&self) -> &dyn Any;

    fn ser(&self) -> Vec<u8>;
}

#[allow(dead_code)]
pub(crate) fn deser(_: &Id, _: &SocketAddr, _: &[u8]) -> Result<Box<dyn Msg>, Error> {
    let mtype: i32 = 0;
    //let reader = Reader::new(cbor);
    //let value: Value = from_reader(reader).unwrap();

    match Kind::from(mtype) {
        Kind::Error => {
            panic!("TODO")
        }
        Kind::Request => match Method::from(mtype) {
            Method::Unknown => {
                panic!("TODO")
            }
            Method::Ping => {
                panic!("TODO")
            }
            Method::FindNode => {
                panic!("TODO")
            }
            Method::AnnouncePeer => {
                panic!("TODO")
            }
            Method::FindPeer => {
                panic!("TODO")
            }
            Method::StoreValue => {
                panic!("TODO")
            }
            Method::FindValue => {
                panic!("TODO")
            }
        },
        Kind::Response => match Method::from(mtype) {
            Method::Unknown => {
                panic!("TODO")
            }
            Method::Ping => {
                panic!("TODO")
            }
            Method::FindNode => {
                panic!("TODO")
            }
            Method::AnnouncePeer => {
                panic!("TODO")
            }
            Method::FindPeer => {
                panic!("TODO")
            }
            Method::StoreValue => {
                panic!("TODO")
            }
            Method::FindValue => {
                panic!("TODO")
            }
        },
    }
}

#[allow(dead_code)]
pub(crate) fn serialize(msg: &Box<dyn Msg>) -> Option<Vec<u8>> {
    Some(msg.ser())
}

pub(crate) fn msg_type(kind: Kind, method: Method) -> i32 {
    kind as i32 | method as i32
}
