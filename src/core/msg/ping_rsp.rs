use std::net::SocketAddr;

use crate::id::Id;
use super::parts::{MsgParts, PartsProxy};

pub(super) struct PingResponseMsg {
    parts: MsgParts
}

#[allow(dead_code)]
impl PingResponseMsg {
    pub(super) fn new(txid: i32) -> Self {
        PingResponseMsg {
            parts: MsgParts::with_txid(txid),
        }
    }
}

impl PartsProxy for PingResponseMsg {
    fn orign_addr(&self) -> &SocketAddr {
        &self.parts.origin
    }

    fn remote_addr(&self) -> &SocketAddr {
        &self.parts.remote
    }

    fn id(&self) -> &Id {
        &self.parts.id
    }

    fn remote_id(&self) -> &Id {
        &self.parts.remote_id
    }

    fn txid(&self) -> i32 {
        self.parts.txid
    }

    fn version(&self) -> i32 {
        self.parts.version
    }

    fn set_orign_addr(&mut self, addr: &SocketAddr) {
        self.parts.origin = *addr;
    }

    fn set_remote_addr(&mut self, addr: &SocketAddr) {
        self.parts.remote = *addr;
    }
    fn set_id(&mut self, id:&Id) {
        self.parts.id = *id;
    }

    fn set_remote_id(&mut self, id: &Id) {
        self.parts.remote_id = *id;
    }

    fn set_txid(&mut self, txid: i32) {
        self.parts.txid = txid;
    }

    fn set_version(&mut self, version: i32) {
        self.parts.version = version;
    }
}

impl ToString for PingResponseMsg {
    fn to_string(&self) -> String {
        // TODO:
        String::new()
    }
}