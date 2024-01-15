use std::rc::Rc;
use std::net::SocketAddr;

use crate::constants;
use crate::id::Id;
use crate::lookup_option::LookupOption;
use crate::nodeinfo::NodeInfo;
use crate::peerinfo::PeerInfo;
use crate::value::Value;
use crate::rpccall::RPCCall;
use crate::rpcserver::RpcServer;
use crate::msg::message::{self, Message, MessageBuidler};
use crate::msg::lookup::{self, ResultBuilder};
use crate::msg::ping::{self};
use crate::msg::find_node::{self};
use crate::kclosest_nodes::KClosestNodes;

pub(crate) struct Task {}

#[allow(dead_code)]
pub(crate) struct DHT {
    persist_root: String,

    rpcserver: Rc<RpcServer>,
}


pub(crate) trait Protocols {

}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(_: &SocketAddr, server: Rc<RpcServer>) -> Self {
        DHT {
            persist_root: "".to_string(),
            rpcserver: server,
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_root = path.to_string()
    }

    pub(crate) fn find_node<F>(&self, _: &Id, _: &LookupOption, _: F) -> Box<Task>
    where F: Fn(&NodeInfo) {

        unimplemented!()
    }

    pub(crate) fn find_value<F>(&self, _: &Id, _: LookupOption, _: F) -> Box<Task>
    where F: Fn(&Value) {
        unimplemented!()
    }

    pub(crate) fn store_value<F>(&self, _: &Value, _: F) -> Box<Task>
    where F: Fn(&[&NodeInfo]) {
        unimplemented!()
    }

    pub(crate) fn find_peer<F>(&self, _: &Id, _: i32, _: &LookupOption, _: F) -> Box<Task>
    where F: Fn(&[&PeerInfo]) {
        unimplemented!()
    }

    pub(crate) fn announce_peer<F>(&self, _: &PeerInfo, _: F) -> Box<Task>
    where F: Fn(&[&NodeInfo]) {
        unimplemented!()
    }

    pub(crate) fn on_timeout(&self, _: &RPCCall) {
        unimplemented!()
    }

    pub(crate) fn on_send(&self, _: &Id) {
        unimplemented!()
    }

    fn on_message(&self, msg: impl Message + lookup::Option) {
        match msg.kind() {
            message::Kind::Error => self.on_request(msg),
            message::Kind::Request => self.on_request(msg),
            message::Kind::Response => self.on_response(msg),
        }
    }

    fn on_request(&self, msg: impl Message + lookup::Option) {
        match msg.method() {
            message::Method::Ping => self.on_ping(msg),
            message::Method::FindNode => self.on_find_node(msg),
            message::Method::FindValue => self.on_find_value(msg),
            message::Method::StoreValue => self.on_store_value(msg),
            message::Method::FindPeer => self.on_find_peers(msg),
            message::Method::AnnouncePeer => self.on_announce_peer(msg),
            message::Method::Unknown => {}
        }
    }

    fn on_response(&self, _: impl Message) {
        unimplemented!()
    }

    fn on_error(&self, _: impl Message) {
        unimplemented!()
    }

    fn on_ping(&self, msg: impl Message) {
        let mut builder = ping::ResponseBuilder::new();
        builder.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr());

        self.rpcserver.send_msg(
            Box::new(builder.build())
        );
    }

    fn on_find_node(&self, msg: impl Message + lookup::Option) {
        let mut builder = find_node::ResponseBuilder::new();
        builder.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr())
            .populate_closest_nodes4(msg.want4(), || {
                KClosestNodes::new(
                    self,
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).as_nodes()
            })
            .populate_closest_nodes6(msg.want6(), || {
                KClosestNodes::new(
                    self,
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).as_nodes()
            })
            .populate_token(msg.want_token(), || {
                let token = 0;
                token
            }
        );

        self.rpcserver.send_msg(
            Box::new(builder.build())
        );
    }

    fn on_find_value(&self, _: impl Message) {
        unimplemented!()
    }

    fn on_store_value(&self, _: impl Message) {
        unimplemented!()
    }

    fn on_find_peers(&self, _: impl Message) {
        unimplemented!()
    }

    fn on_announce_peer(&self, _: impl Message) {
        unimplemented!()
    }
}
