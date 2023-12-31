
use std::net::SocketAddr;
use crate::id::Id;

pub(crate) struct CommonFields {
    pub(crate) origin: SocketAddr,
    pub(crate) remote: SocketAddr,
    pub(crate) id: Id,
    pub(crate) remote_id: Id,
    // associated rpc,

    pub(crate) txid: i32,
    pub(crate) version: i32,
}