use std::net::SocketAddr;

use crate::id::Id;
use crate::msg::message::{MsgGetter, MsgSetter};
use crate::msg::common::CommonFields;

pub(crate) struct ErrorMsg {
    common: CommonFields
}

impl MsgGetter for ErrorMsg {
    fn orign(&self) -> &SocketAddr {
        &self.common.origin
    }

    fn remote_addr(&self) -> &SocketAddr {
        &self.common.remote
    }

    fn id(&self) -> &Id {
        &self.common.id
    }

    fn remote_id(&self) -> &Id {
        &self.common.remote_id
    }

    fn txid(&self) -> i32 {
        self.common.txid
    }

    fn version(&self) -> i32 {
        self.common.version
    }
}

impl MsgSetter for ErrorMsg {
    fn set_orign(&mut self, addr: &SocketAddr) {
        self.common.origin = *addr;
    }

    fn set_remote_addr(&mut self, addr: &SocketAddr) {
        self.common.remote = *addr;
    }
    fn set_id(&mut self, id:&Id) {
        self.common.id = *id;
    }

    fn set_remote_id(&mut self, id: &Id) {
        self.common.remote_id = *id;
    }

    fn set_txid(&mut self, txid: i32) {
        self.common.txid = txid;
    }

    fn set_version(&mut self, version: i32) {
        self.common.version = version;
    }
}

impl ToString for ErrorMsg {
    fn to_string(&self) -> String {
        // TODO:
        String::new()
    }
}