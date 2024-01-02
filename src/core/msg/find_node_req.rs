use std::net::SocketAddr;

use crate::id::Id;
use super::parts::{MsgParts, PartsProxy};
use super::lookup::{Filters, FilterProxy};

pub(super) struct FindNodeRequestMsg {
    parts: MsgParts,
    filters: Filters
}

#[allow(dead_code)]
impl FindNodeRequestMsg {
    pub(super) fn new() -> Self {
        FindNodeRequestMsg {
            parts: MsgParts::new(),
            filters: Filters::new(),
        }
    }
}

impl PartsProxy for FindNodeRequestMsg {
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

impl FilterProxy for FindNodeRequestMsg {
    fn target(&self) -> &Id {
        &self.filters.target
    }

    fn does_want4(&self) -> bool {
        self.filters.want4
    }

    fn does_want6(&self) -> bool {
        self.filters.want6
    }

    fn set_want4(&mut self) {
        self.filters.want4 = true;
    }

    fn set_want6(&mut self) {
        self.filters.want6 = true
    }
}

impl ToString for FindNodeRequestMsg {
    fn to_string(&self) -> String {
        // TODO:
        String::new()
    }
}
