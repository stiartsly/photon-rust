use std::net::SocketAddr;

use ciborium::{de::from_reader, Value};
use ciborium_io::Read;
use core::result::Result;
use std::io::{Error};

use crate::id::Id;
use super::ping::{self};

#[allow(dead_code)]
pub(crate) enum Kind {
    Error = 0x00,
    Request = 0x20,
    Response = 0x40,
}

#[allow(dead_code)]
impl Kind {
    const MASK: i32 = 0xE0;
    fn from(mtype: i32) -> Kind {
        let kind: i32 = mtype & Self::MASK;
        match kind {
            0x00 => Kind::Error,
            0x20 => Kind::Request,
            0x40 => Kind::Response,
            _ => {panic!("invalid msg kind: {}", kind)}
        }
    }
}

#[allow(dead_code)]
pub(crate) enum Method {
    Unknown = 0x00,
    Ping = 0x01,
    FindNode = 0x02,
    AnnouncePeer = 0x03,
    FindPeer = 0x04,
    StoreValue = 0x05,
    FindValue = 0x6
}

#[allow(dead_code)]
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
            _ => {panic!("invalid msg method: {}", method)}
        }
    }
}

pub(crate) trait Message {
    fn kind(&self) -> Kind;
    fn method(&self) -> Method;

    fn id(&self) -> &Id;
    fn addr(&self) -> &SocketAddr;

    fn txid(&self) -> i32;
    fn version(&self) -> i32;
}

pub(crate) trait MessageBuidler<'a> {
    fn with_id(&mut self, _: &'a Id) -> &mut Self;
    fn with_addr(&mut self, _: &'a SocketAddr) -> &mut Self;

    fn with_txid(&mut self, _: i32) -> &mut Self;
    fn with_verion(&mut self, _: i32) -> &mut Self;
}

pub(crate) trait MessageParser<'a> {
    fn with_cbor(&mut self, _: &'a [u8]) -> &mut Self;
}

#[allow(dead_code)]
pub(crate) fn deser(_: &Id, _: &SocketAddr, cbor: &[u8]) -> Box<dyn Message> {
    let mtype: i32 = 0;
    let reader = Reader::new(cbor);
    let value: Value = from_reader(reader).unwrap();

    match Kind::from(mtype) {
        Kind::Error => { panic!("TODO") },
        Kind::Request => {
            match Method::from(mtype) {
                Method::Unknown => { panic!("TODO") },
                Method::Ping => Box::new(ping::RequestBuidler::from(&value).build()),
                Method::FindNode => { panic!("TODO") },
                Method::AnnouncePeer => { panic!("TODO") },
                Method::FindPeer => { panic!("TODO") },
                Method::StoreValue => { panic!("TODO") },
                Method::FindValue => { panic!("TODO") }
            }
        },
        Kind::Response => {
            match Method::from(mtype) {
                Method::Unknown => { panic!("TODO") },
                Method::Ping => Box::new(ping::ResponseBuilder::from(&value).build()),
                Method::FindNode => { panic!("TODO") },
                Method::AnnouncePeer => { panic!("TODO") },
                Method::FindPeer => { panic!("TODO") },
                Method::StoreValue => { panic!("TODO") },
                Method::FindValue => { panic!("TODO") }
            }
        }
    }
}

#[allow(dead_code)]
pub(crate) fn serilize(_: Box<dyn Message>) -> Vec<u8> {
    unimplemented!()
}

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

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
            // If not enough data is remaining, return an error
            Err(Error::from(std::io::ErrorKind::UnexpectedEof))
        }
    }
}
