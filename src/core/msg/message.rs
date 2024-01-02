use std::net::SocketAddr;
use std::boxed::Box;

use crate::id::Id;

#[allow(dead_code)]
pub(crate) enum MsgType {
    Error = 0x00,
    Request = 0x20,
    Response = 0x40
}

#[allow(dead_code)]
pub(crate) enum MsgMethod {
    Unknown = 0x00,
    Ping = 0x01,
    FindNode = 0x02,
    AnnouncePeer = 0x03,
    FindPeer = 0x04,
    StoreValue = 0x05,
    FindValue = 0x6
}

pub(crate) trait Message {
    fn mtype(&self) -> MsgType;
    fn method(&self) -> MsgMethod;

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

    fn is_valid(&self) -> bool;
}

#[allow(dead_code)]
pub(crate) fn deserialize(_: &Id, _: &SocketAddr, _: &[u8]) -> Box<dyn Message> {
    unimplemented!()
}

#[allow(dead_code)]
pub(crate) fn serialize(_: Box<dyn Message>) -> Vec<u8> {
    unimplemented!()
}