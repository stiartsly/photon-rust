use std::rc::Rc;
use std::net::SocketAddr;

use crate::constants;
use crate::id::Id;
use crate::lookup_option::LookupOption;
use crate::node::Node;
use crate::peer::Peer;
use crate::value::{Value};
use crate::rpccall::RPCCall;
use crate::rpcserver::RpcServer;
use crate::msg::message::{self, Message, MessageBuidler};
use crate::msg::lookup::{self, ResultBuilder};
use crate::msg::ping::{self};
use crate::msg::find_node::{self};
use crate::msg::find_value::{self, ValueResultBuilder};
use crate::msg::find_peer::{self, PeerResultBuilder};
use crate::msg::store_value::{self};
use crate::msg::announce_peer::{self};
use crate::msg::error_msg::{self, ErrorResult, ErrorResultBuilder};
use crate::kclosest_nodes::KClosestNodes;
use crate::token_manager::TokenManager;
use crate::routing_table::RoutingTable;
use crate::version;

use log::{info, warn, debug};

pub(crate) struct Task {}

#[allow(dead_code)]
pub(crate) struct DHT {
    addr: SocketAddr,
    persist_root: String,

    routing_table: Box<RoutingTable>,
    rpcserver: Rc<RpcServer>,
    token_manager: TokenManager,

    running: bool,
}


pub(crate) trait Protocols {

}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(addr: &SocketAddr, server: Rc<RpcServer>) -> Self {
        DHT {
            addr: addr.clone(),
            persist_root: "".to_string(),
            rpcserver: server,
            routing_table: Box::new(RoutingTable::new()),
            token_manager: TokenManager::new(),
            running: false,
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_root = path.to_string()
    }

    pub(crate) fn find_node<F>(&self, _: &Id, _: &LookupOption, _: F) -> Box<Task>
    where F: Fn(&Node) {

        unimplemented!()
    }

    pub(crate) fn find_value<F>(&self, _: &Id, _: LookupOption, _: F) -> Box<Task>
    where F: Fn(&Value) {
        unimplemented!()
    }

    pub(crate) fn store_value<F>(&self, _: &Value, _: F) -> Box<Task>
    where F: Fn(&[&Node]) {
        unimplemented!()
    }

    pub(crate) fn find_peer<F>(&self, _: &Id, _: i32, _: &LookupOption, _: F) -> Box<Task>
    where F: Fn(&[&Peer]) {
        unimplemented!()
    }

    pub(crate) fn announce_peer<F>(&self, _: &Peer, _: F) -> Box<Task>
    where F: Fn(&[&Node]) {
        unimplemented!()
    }

    pub(crate) fn on_timeout(&self, call: &RPCCall) {
        // ignore the timeout if the DHT is stopped or the RPC server is offline
        if !self.running || !self.rpcserver.is_reachable() {
            return;
        }

        self.routing_table.on_timeout(call.id());
    }

    pub(crate) fn on_send(&self, id: &Id) {
        if !self.running {
            return;
        }

        self.routing_table.on_send(id)
    }

    fn on_message<T>(&self, msg: T )
    where T: Message + lookup::Option + find_value::ValueOption + store_value::StoreOption +
        announce_peer::AnnounceOption + error_msg::ErrorResult{
        match msg.kind() {
            message::Kind::Error => self.on_error(msg),
            message::Kind::Request => self.on_request(msg),
            message::Kind::Response => self.on_response(msg),
        }
    }

    fn on_request<T>(&self, msg: T)
    where T: Message + lookup::Option + find_value::ValueOption + store_value::StoreOption +
        announce_peer::AnnounceOption {
        match msg.method() {
            message::Method::Ping => self.on_ping(msg),
            message::Method::FindNode => self.on_find_node(msg),
            message::Method::FindValue => self.on_find_value(msg),
            message::Method::StoreValue => self.on_store_value(msg),
            message::Method::FindPeer => self.on_find_peers(msg),
            message::Method::AnnouncePeer => self.on_announce_peer(msg),
            message::Method::Unknown => {
                self.send_err(msg, 203, "Invalid request method");
            }
        }
    }

    fn on_response(&self, _: impl Message) {}

    fn on_error(&self, msg: impl Message + ErrorResult) {
        warn!("Error from {}/{} - {}:{}, txid {}",
            msg.addr(),
            version::readable_version(msg.version()),
            msg.code(),
            msg.msg(),
            msg.txid()
        );
    }

    fn send_err<'a>(&self, msg: impl Message, code: i32, str: &'a str) {
        let mut b = error_msg::ErrorMsgBuilder::new();
        b.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr())
            .with_msg(str)
            .with_code(code);
        self.rpcserver.send_msg(Box::new(b.build()));
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

    fn on_find_node<T>(&self, msg: T) where T: Message + lookup::Option {
        let mut b = find_node::ResponseBuilder::new();
        b.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr())
            .populate_closest_nodes4(msg.want4(), || {
                KClosestNodes::new(
                    self, // TODO: about DHT reference
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).fill(true).as_nodes()
            })
            .populate_closest_nodes6(msg.want6(), || {
                KClosestNodes::new(
                    self, // TODO: about DHT reference
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).fill(true).as_nodes()
            })
            .populate_token(msg.want_token(), || {
                self.token_manager.generate_token()
            }
        );

        self.rpcserver.send_msg(Box::new(b.build()))
    }

    fn on_find_value<T>(&self, msg: T)
    where T: Message + lookup::Option + find_value::ValueOption {
        let mut b = find_value::ResponseBuilder::new();
        let mut has_value = false;
        b.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr())
            .populate_value(|| {
                // TODO;
                let value: Option<Box<Value>> = None;
                if value.is_some() {
                    if msg.seq() < 0 || value.as_ref().unwrap().sequence_number() < 0
                        || msg.seq() <= value.as_ref().unwrap().sequence_number() {
                        has_value = true;
                    }
                }
                value
            })
            .populate_closest_nodes4(msg.want4() && has_value, || {
                KClosestNodes::new(self,
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).fill(true).as_nodes()
            })
            .populate_closest_nodes6(msg.want6() && has_value, || {
                KClosestNodes::new(self,
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).fill(true).as_nodes()
            })
            .populate_token(msg.want_token(), || {
                self.token_manager.generate_token()
            }
        );

        self.rpcserver.send_msg(Box::new(b.build()));
    }

    fn on_store_value<T>(&self, msg: T)
    where T: Message + lookup::Option + store_value::StoreOption {
        let value = msg.value();
        let value_id = value.id();

        if !self.token_manager.verify_token(msg.token(), msg.id(), msg.addr(), &value_id) {
            warn!("Received a store value request with invalid token from {}", msg.addr());
            self.send_err(msg, 203, "Invalid token for STORE VALUE request");
            return;
        }

        if value.is_valid().is_err() {
            self.send_err(msg, 203, "Invalid value");
            return;
        }
        // TODO: store value.
        let mut b = store_value::ResponseBuilder::new();
        b.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr());

        self.rpcserver.send_msg(Box::new(b.build()));
    }

    fn on_find_peers<T>(&self, msg: T)
    where T: Message + lookup::Option + find_value::ValueOption  {
        let mut b = find_peer::ResponseBuilder::new();
        let mut has_peers = false;
        b.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr())
            .populate_peers(|| {
                // TODO;
                let peers: Vec<Box<Peer>> = Vec::new();
                if !peers.is_empty() {
                    has_peers = true;
                };
                peers
            })
            .populate_closest_nodes4(msg.want4() && has_peers, || {
                KClosestNodes::new(self,
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).fill(true).as_nodes()
            })
            .populate_closest_nodes6(msg.want6() && has_peers, || {
                KClosestNodes::new(self,
                    msg.target(),
                    constants::MAX_ENTRIES_PER_BUCKET
                ).fill(true).as_nodes()
            })
            .populate_token(msg.want_token(), || {
                self.token_manager.generate_token()
            }
        );

        self.rpcserver.send_msg(Box::new(b.build()));
    }

    fn on_announce_peer<T>(&self, msg: T)
    where T: Message + lookup::Option + announce_peer::AnnounceOption {
        let bogon = false;

        if bogon {
            info!("Received an announce peer request from bogon address {}, ignored ", msg.addr());
        }

        if !self.token_manager.verify_token(msg.token(), msg.id(), msg.addr(), msg.target()) {
            warn!("Received an announce peer request with invalid token from {}", msg.addr());
            self.send_err(msg, 203, "Invalid token for ANNOUNCE PEER request");
            return;
        }

        let peers = msg.peers();
        for peer in peers.iter() {
            if peer.is_valid().is_err() {
                self.send_err(msg, 203, "One peer is invalid peer");
                return;
            }
        };

        debug!("Received an announce peer request from {}, saving peer {}", msg.addr(), msg.target());
        // TODO: Store peers.

        let mut b = announce_peer::ResponseBuilder::new();
        b.with_txid(msg.txid())
            .with_id(msg.id())
            .with_addr(msg.addr());

        self.rpcserver.send_msg(Box::new(b.build()));
        unimplemented!()
    }
}
