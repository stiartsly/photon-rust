
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use crate::id::Id;

pub(super) struct MsgParts {
    pub(super) origin: SocketAddr,
    pub(super) remote: SocketAddr,
    pub(super) id: Id,
    pub(super) remote_id: Id,
    // associated rpc,

    pub(super) txid: i32,
    pub(super) version: i32,
}

pub(super) trait PartsProxy {
    fn remote_addr(&self) -> &SocketAddr;
    fn orign_addr(&self) -> &SocketAddr;
    fn id(&self) -> &Id;
    fn remote_id(&self) -> &Id;
    fn txid(&self) -> i32;
    fn version(&self) -> i32;

    fn set_orign_addr(&mut self, addr: &SocketAddr);
    fn set_remote_addr(&mut self, addr: &SocketAddr);
    fn set_id(&mut self, id:&Id);
    fn set_remote_id(&mut self, id: &Id);
    fn set_txid(&mut self, txid: i32);
    fn set_version(&mut self, version: i32);
}

#[allow(dead_code)]
impl MsgParts {
    pub(super) fn new() -> Self {
        MsgParts {
            origin: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            remote: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            id: Id::random(),
            remote_id: Id::random(),
            txid: 0,
            version: 0
        }
    }

    pub(super) fn with_txid(txid: i32) -> Self {
        MsgParts {
            origin: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            remote: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            id: Id::random(),
            remote_id: Id::random(),
            txid: txid,
            version: 0
        }
    }
}
