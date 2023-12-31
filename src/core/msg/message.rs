use std::net::SocketAddr;
use crate::id::Id;
use crate::msg::error::ErrorMsg;

enum Method {

}

pub(crate) enum Message {
    Error(ErrorMsg),

}

pub(crate) trait MsgGetter {
    fn remote_addr(&self) -> &SocketAddr;
    fn orign(&self) -> &SocketAddr;
    fn id(&self) -> &Id;
    fn remote_id(&self) -> &Id;
    fn txid(&self) -> i32;
    fn version(&self) -> i32;
}

pub(crate) trait MsgSetter {
    fn set_orign(&mut self, addr: &SocketAddr);
    fn set_remote_addr(&mut self, addr: &SocketAddr);
    fn set_id(&mut self, id:&Id);
    fn set_remote_id(&mut self, id: &Id);
    fn set_txid(&mut self, txid: i32);
    fn set_version(&mut self, version: i32);
}
