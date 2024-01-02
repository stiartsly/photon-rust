use std::net::SocketAddr;

use crate::id::Id;
use super::parts::{MsgParts, PartsProxy };

pub(super) struct PingRequestMsg {
    parts: MsgParts
}

#[allow(dead_code)]
impl PingRequestMsg {
    pub(super) fn new() -> Self {
        PingRequestMsg {
            parts: MsgParts::new(),
        }
    }
}

impl PartsProxy for PingRequestMsg {
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

impl ToString for PingRequestMsg {
    fn to_string(&self) -> String {
        // TODO:
        String::new()
    }
}