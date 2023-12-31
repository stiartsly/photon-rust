
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use crate::id::Id;

pub(crate) struct Fields {
    pub(crate) origin: SocketAddr,
    pub(crate) remote: SocketAddr,
    pub(crate) id: Id,
    pub(crate) remote_id: Id,
    // associated rpc,

    pub(crate) txid: i32,
    pub(crate) version: i32,
}

pub(crate) trait Get {
    fn remote_addr(&self) -> &SocketAddr;
    fn orign(&self) -> &SocketAddr;
    fn id(&self) -> &Id;
    fn remote_id(&self) -> &Id;
    fn txid(&self) -> i32;
    fn version(&self) -> i32;
}

pub(crate) trait Set {
    fn set_orign(&mut self, addr: &SocketAddr);
    fn set_remote_addr(&mut self, addr: &SocketAddr);
    fn set_id(&mut self, id:&Id);
    fn set_remote_id(&mut self, id: &Id);
    fn set_txid(&mut self, txid: i32);
    fn set_version(&mut self, version: i32);
}

impl Fields {
    pub(crate) fn new() -> Self {
        Fields {
            origin: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            remote: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            id: Id::random(),
            remote_id: Id::random(),
            txid: 0,
            version: 0
        }
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Fields {
            origin: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            remote: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            id: Id::random(),
            remote_id: Id::random(),
            txid: txid,
            version: 0
        }
    }
}
