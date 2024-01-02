use std::net::SocketAddr;

use crate::id::Id;
use crate::node_info::NodeInfo;
use super::parts::{MsgParts, PartsProxy};
use crate::msg::lookup::{Results, ResultProxy};

pub(super) struct FindNodeResponseMsg {
    parts: MsgParts,
    results: Results,
}

#[allow(dead_code)]
impl FindNodeResponseMsg {
    pub(super) fn new() -> Self {
        FindNodeResponseMsg {
            parts: MsgParts::new(),
            results: Results::new()
        }
    }
}

impl PartsProxy for FindNodeResponseMsg {
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

impl ResultProxy for FindNodeResponseMsg {
    fn nodes4(&self) -> &[NodeInfo] {
        &self.results.nodes4
    }

    fn nodes6(&self) -> &[NodeInfo] {
        &self.results.nodes6
    }

    fn token(&self) -> i32 {
        self.results.token
    }

    fn set_nodes4(&self, _: &[NodeInfo]) {
        // TODO:
    }
    fn set_nodes6(&self, _: &[NodeInfo]) {
        // TODO;
    }
    fn set_token(&self, _: i32) {
        // TODO;
    }
}

impl ToString for FindNodeResponseMsg {
    fn to_string(&self) -> String {
        // TODO:
        String::new()
    }
}
