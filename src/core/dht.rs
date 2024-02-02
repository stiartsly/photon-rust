use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;

use crate::constants;
use crate::version;
use crate::id::Id;
use crate::lookup_option::LookupOption;
use crate::node::Node;
use crate::peer::Peer;
use crate::value::Value;
use crate::rpccall::RpcCall;
use crate::rpcserver::RpcServer;
use crate::kclosest_nodes::KClosestNodes;
use crate::token_man::TokenManager;
use crate::routing_table::RoutingTable;
use crate::msg::{
    msg::{self, Msg},
    lookup::{self, Result},
    error::{self, ErrorResult},
    ping_req,
    find_node_rsp,
    find_value_req,
    find_value_rsp::{self, ValueResult},
    find_peer_rsp::{self, PeerResult},
    store_value_req,
    store_value_rsp,
    announce_peer_req,
    announce_peer_rsp
};
use crate::task::{
    task::Task,
    task_manager::TaskManager,
    node_lookup::NodeLookupTaskBuilder,
    value_lookup::ValueLookupTaskBuilder,
    peer_lookup::PeerLookupTaskBuilder
};

use log::{info, warn, debug};

#[allow(dead_code)]
pub(crate) struct DHT {
    // node: Rc<NodeRunner>,

    server: Rc<RpcServer>,
    token_man: TokenManager,

    addr: SocketAddr,

    routing_table: Box<RoutingTable>,
    task_manager: TaskManager,

    persist_path: String,
    running: bool,
}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(binding_addr: &SocketAddr, server: Rc<RpcServer>) -> Self {
        DHT { server,
            token_man: TokenManager::new(),

            addr: binding_addr.clone(),
            persist_path: "".to_string(),

            routing_table: Box::new(RoutingTable::new()),
            task_manager: TaskManager::new(),
            running: false,
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_path = path.to_string()
    }

    pub(crate)fn bootstrap() {
        unimplemented!()
    }

    pub(crate) fn find_node<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Box<Node>>) + 'static {
        let mut entry = self.routing_table.bucket_entry(id).map(
            |entry| Box::new(entry.node().clone())
        );

        let result = Rc::new(RefCell::new(entry.take()));
        let result_shadow = Rc::clone(&result);

        let mut builder = NodeLookupTaskBuilder::new(id);
        builder.with_name("node lookup");
        builder.set_result_fn(move|arg_task, arg_node| {
            if arg_node.is_some() {
                let mut found_borrowed = result.borrow_mut();
                *found_borrowed = Some(arg_node.unwrap());
            }
            if option == LookupOption::Conservative {
                arg_task.cancel()
            }
        });
        let mut task = Box::new(builder.build());
        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        });

        self.task_manager.add(task as Box<dyn Task>);
    }

    pub(crate) fn find_value<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Box<Value>>) + 'static {
        let mut empty: Option<Box<Value>> = None;
        let result_ref = Rc::new(RefCell::new(empty.take()));
        let result_shadow = Rc::clone(&result_ref);

        let mut builder = ValueLookupTaskBuilder::new(id);
        builder.with_name("value lookup");
        builder.set_result_fn(move | arg_task, arg_value| {
            let mut found_borrowed = result_ref.borrow_mut();
            let value_ref = arg_value.as_ref().unwrap();
            match result_ref.borrow().as_ref() {
                Some(value) => {
                    if arg_value.as_ref().unwrap().is_mutable() &&
                        value.sequence_number() < value_ref.sequence_number() {
                        *found_borrowed = Some(value_ref.clone());
                    }
                },
                None => {
                    *found_borrowed = Some(value_ref.clone())
                }
            }
            if option != LookupOption::Conservative || value_ref.is_mutable() {
                arg_task.cancel()
            }
        });

        let mut task = Box::new(builder.build());
        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        });

        self.task_manager.add(Box::new(builder.build()));
    }

    pub(crate) fn store_value<F>(&self, _: &Value, _: F)
    where F: Fn(Option<Vec<Box<Node>>>) + 'static {
        unimplemented!()
    }

    pub(crate) fn find_peer<F>(&self, id: &Id, expected: usize, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Vec<Box<Peer>>>) + 'static {
        let mut empty: Option<Vec<Box<Peer>>> = None;
        let result_rc = Rc::new(RefCell::new(empty.take()));
        let result_shadow = Rc::clone(&result_rc);

        let mut builder = PeerLookupTaskBuilder::new(id);
        builder.with_name("peer-lookup");
        builder.set_result_fn(move |arg_task, _| {
            let found_borrowed = result_rc.borrow_mut();
            // peers->insert(peers->end(), listOfPeers.begin(), listOfPeers.end());
            if option != LookupOption::Conservative && (*found_borrowed).as_ref().unwrap().len() >= expected {
                arg_task.cancel();
                return;
            }
        });

        let mut task = Box::new(builder.build());
        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take())
        });

        self.task_manager.add(task as Box<dyn Task>);
    }

    pub(crate) fn announce_peer<F>(&self, _: &Peer, _: F)
    where F: Fn(&[&Node]) {
        unimplemented!()
    }

    pub(crate) fn on_timeout(&self, call: &RpcCall) {
        // ignore the timeout if the DHT is stopped or the RPC server is offline
        if !self.running || !self.server.is_reachable() {
            return;
        }
        self.routing_table.on_timeout(call.target_id());
    }

    pub(crate) fn on_send(&self, id: &Id) {
        if !self.running {
            return;
        }
        self.routing_table.on_send(id)
    }

    fn on_message<T>(&mut self, msg: &Box<T> )
    where T: Msg + lookup::Condition + find_value_req::ValueOption + store_value_req::StoreOption +
        announce_peer_req::AnnounceOption + error::ErrorResult{
        match msg.kind() {
            msg::Kind::Error => self.on_error(msg),
            msg::Kind::Request => self.on_request(msg),
            msg::Kind::Response => self.on_response(msg.as_ref()),
        }
    }

    fn on_request<T>(&mut self, msg: &Box<T>)
    where T: Msg + lookup::Condition + find_value_req::ValueOption + store_value_req::StoreOption +
        announce_peer_req::AnnounceOption {
        match msg.method() {
            msg::Method::Ping => self.on_ping(msg.as_ref()),
            msg::Method::FindNode => self.on_find_node(msg),
            msg::Method::FindValue => self.on_find_value(msg),
            msg::Method::StoreValue => self.on_store_value(msg),
            msg::Method::FindPeer => self.on_find_peers(msg),
            msg::Method::AnnouncePeer => self.on_announce_peer(msg),
            msg::Method::Unknown => {
                self.send_err(msg.as_ref(), 203, "Invalid request method");
            }
        }
    }

    fn on_response(&self, _: &dyn Msg) {}
    fn on_error<T>(&self, msg: &Box<T>) where T: Msg + error::ErrorResult{
        warn!("Error from {}/{} - {}:{}, txid {}",
            msg.addr(),
            version::readable_version(msg.version()),
            msg.code(),
            msg.msg(),
            msg.txid()
        );
    }

    fn send_err(&self, msg: &dyn Msg, code: i32, str: &str) {
        let mut err = Box::new(error::Message::new());

        err.with_id(msg.id());
        err.with_txid(msg.txid());
        err.with_addr(msg.addr());
        err.with_msg(str);
        err.with_code(code);

        self.server.send_msg(err);
    }


    fn on_ping(&self, request: &dyn Msg) {
        let mut msg = Box::new(ping_req::Message::new());

        msg.with_id(request.id());
        msg.with_txid(request.txid());
        msg.with_addr(request.addr());

        self.server.send_msg(msg);
    }

    fn on_find_node<T>(&self, request: &Box<T>) where T: Msg + lookup::Condition {
        let mut resp = Box::new(find_node_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        resp.populate_closest_nodes4(request.want4(), || {
            Some(KClosestNodes::new(
                Rc::new(self),
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_closest_nodes6(request.want6(), || {
            Some(KClosestNodes::new(
                Rc::new(self),
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_token(request.want_token(), || {
                self.token_man.generate_token(
                    request.id(),
                    request.addr(),
                    request.target()
                )
            }
        );

        self.server.send_msg(resp)
    }

    fn on_find_value<T>(&self, request: &Box<T>)
    where T: Msg + lookup::Condition + find_value_req::ValueOption {
        let mut resp = Box::new(find_value_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        let mut has_value = false;
        resp.populate_value(|| {
            // TODO;
            let value: Option<Box<Value>> = None;
            if value.is_some() {
                if request.seq() < 0 || value.as_ref().unwrap().sequence_number() < 0
                    || request.seq() <= value.as_ref().unwrap().sequence_number() {
                    has_value = true;
                }
            }
            value
        });

        resp.populate_closest_nodes4(request.want4() && has_value, || {
            Some(KClosestNodes::new(
                Rc::new(&self),
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_closest_nodes6(request.want6() && has_value, || {
            Some(KClosestNodes::new(
                Rc::new(&self),
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_token(request.want_token(), || {
            self.token_man.generate_token(
                request.id(),
                request.addr(),
                request.target()
            )
        });

        self.server.send_msg(resp);
    }

    fn on_store_value<T>(&mut self, request: &Box<T>)
    where T: Msg + lookup::Condition + store_value_req::StoreOption {
        let value = request.value();
        let value_id = value.id();

        if !self.token_man.verify_token(request.token(), request.id(), request.addr(), &value_id) {
            warn!("Received a store value request with invalid token from {}", request.addr());
            self.send_err(request.as_ref(), 203, "Invalid token for STORE VALUE request");
            return;
        }

        if !value.is_valid() {
            self.send_err(request.as_ref(), 203, "Invalid value");
            return;
        }
        // TODO: store value.
        let mut resp = Box::new(store_value_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        self.server.send_msg(resp);
    }

    fn on_find_peers<T>(&self, request: &Box<T>)
    where T: Msg + lookup::Condition + find_value_req::ValueOption  {
        let mut resp = Box::new(find_peer_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        let mut has_peers = false;
        resp.populate_peers(|| {
            // TODO;
            let peers: Vec<Box<Peer>> = Vec::new();
            if !peers.is_empty() {
                has_peers = true;
            };
            Some(peers)
        });

        resp.populate_closest_nodes4(request.want4() && has_peers, || {
            Some(KClosestNodes::new(
                Rc::new(&self),
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_closest_nodes6(request.want6() && has_peers, || {
            Some(KClosestNodes::new(
                Rc::new(&self),
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_token(request.want_token(), || {
            self.token_man.generate_token(
                request.id(),
                request.addr(),
                request.target()
            )
        });

        self.server.send_msg(resp);
    }

    fn on_announce_peer<T>(&mut self, request: &Box<T>)
    where T: Msg + lookup::Condition + announce_peer_req::AnnounceOption {
        let bogon = false;

        if bogon {
            info!("Received an announce peer request from bogon address {}, ignored ",
                request.addr()
            );
        }

        if !self.token_man.verify_token(
            request.token(),
            request.id(),
            request.addr(),
            request.target()
        ) {
            warn!("Received an announce peer request with invalid token from {}", request.addr());
            self.send_err(request.as_ref(), 203, "Invalid token for ANNOUNCE PEER request");
            return;
        }

        let peers = request.peers();
        for peer in peers.iter() {
            if !peer.is_valid() {
                self.send_err(request.as_ref(), 203, "One peer is invalid peer");
                return;
            }
        };

        debug!("Received an announce peer request from {}, saving peer {}",
            request.addr(),
            request.target()
        );
        // TODO: Store peers.

        let mut resp = Box::new(announce_peer_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        self.server.send_msg(resp);
    }
}
