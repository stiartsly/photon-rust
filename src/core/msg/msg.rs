use std::any::Any;
use std::fmt;
use std::net::SocketAddr;

//use ciborium::{de::from_reader, Value};
use ciborium_io::Read;
use core::result::Result;
use std::io::Error;

use crate::id::Id;
use crate::rpccall::RpcCall;

#[derive(PartialEq)]
pub(crate) enum Kind {
    Error = 0x00,
    Request = 0x20,
    Response = 0x40,
}

impl Kind {
    const MASK: i32 = 0xE0;
    fn from(mtype: i32) -> Kind {
        let kind: i32 = mtype & Self::MASK;
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
    fn from(_type: i32) -> Self {
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

    fn txid(&self) -> i32;
    fn version(&self) -> i32;

    fn with_id(&mut self, _: &Id);
    fn with_addr(&mut self, _: &SocketAddr);

    fn with_txid(&mut self, _: i32);
    fn with_ver(&mut self, _: i32);

    //fn with_cbor(&mut self, _: &[u8]);

    fn associated_call(&self) -> Option<Box<RpcCall>>;
    fn with_associated_call(&mut self, _: Box<RpcCall>);

    fn as_any(&self) -> &dyn Any;
}

#[allow(dead_code)]
pub(crate) fn deser(_: &Id, _: &SocketAddr, _: &[u8]) -> Box<dyn Msg> {
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
pub(crate) fn serilize(_: Box<dyn Msg>) -> Vec<u8> {
    unimplemented!()
}

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

#[allow(dead_code)]
impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Reader { data, position: 0 }
    }
}

impl<'a> Read for Reader<'a> {
    type Error = Error;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let remaining_len = self.data.len() - self.position;

        if remaining_len >= buf.len() {
            // If there is enough data remaining, copy it to buf
            buf.copy_from_slice(&self.data[self.position..self.position + buf.len()]);
            self.position += buf.len();
            Ok(())
        } else {
            Err(Error::from(std::io::ErrorKind::UnexpectedEof))
        }
    }
}
