use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;

use crate::constants;
use crate::id::Id;
use crate::lookup_option::LookupOption;
use crate::node::Node;
use crate::peer::Peer;
use crate::value::{Value};
use crate::rpccall::RpcCall;
use crate::rpcserver::RpcServer;
use crate::kclosest_nodes::KClosestNodes;
use crate::token_manager::TokenManager;
use crate::routing_table::RoutingTable;
use crate::version;
use crate::msg::{
    msg::{self, Msg},
    lookup::{self, Result},
    error::{self, ErrorResult},
    ping_req::{self},
    find_node_rsp::{self},
    find_value_req::{self},
    find_value_rsp::{self, ValueResult},
    find_peer_rsp::{self, PeerResult},
    store_value_req::{self},
    store_value_rsp::{self},
    announce_peer_req::{self},
    announce_peer_rsp::{self}
};
use crate::task::{
    task::{self, Task},
    task_manager::TaskManager,
    node_lookup::NodeLookupTask,
    value_lookup::ValueLookupTask,
    peer_lookup::PeerLookupTask
};

use log::{info, warn, debug};

#[allow(dead_code)]
pub(crate) struct DHT {
    addr: SocketAddr,
    persist_root: String,

    routing_table: Box<RoutingTable>,
    rpcserver: Rc<RpcServer>,
    token_manager: TokenManager,
    task_manager: TaskManager,

    running: bool,
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
            task_manager: TaskManager::new(),
            running: false,
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_root = path.to_string()
    }

    pub(crate) fn find_node<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(&Option<Box<Node>>) + 'static {
        let mut found: Option<Box<Node>> = None;
        match self.routing_table.bucket_entry(id) {
            Some(e) => {
                found = Some(Box::new(e.node().clone()));
            },
            None => {}
        }

        let found_rc = Rc::new(RefCell::new(found.take()));
        let found_shadow = Rc::clone(&found_rc);

        let mut task = Box::new(NodeLookupTask::new(id));
        task.with_name("node lookup");

        task.set_result_fn(move|_node, task| {
            if _node.is_some() {
                let mut found_borrowed = found_rc.borrow_mut();
                *found_borrowed = Some(_node.as_ref().unwrap().clone());
            }
            if option == LookupOption::CONSERVATIVE {
                task.cancel()
            }
        });

        task.add_listener(move |_| {
            complete_fn(&found_shadow.borrow().clone());
        });

        self.task_manager.add(task as Box<dyn Task>);
    }

    pub(crate) fn find_value<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(&Option<Box<Value>>) + 'static {
        let mut found: Option<Box<Value>> = None;
        let mut task = Box::new(ValueLookupTask::new(id));

        let found_rc = Rc::new(RefCell::new(found.take()));
        let found_shadow = Rc::clone(&found_rc);

        task.with_name("value lookup");
        task.set_result_fn(move | value_arg, task_arg| {
            let mut found_borrowed = found_rc.borrow_mut();
            match found_rc.borrow().as_ref() {
                Some(value) => {
                    if value_arg.as_ref().unwrap().is_mutable() &&
                        value.sequence_number() < value_arg.as_ref().unwrap().sequence_number() {
                        *found_borrowed = Some(value_arg.as_ref().unwrap().clone());
                    }
                },
                None => {
                    *found_borrowed = Some(value_arg.as_ref().unwrap().clone())
                }
            }
            if option != LookupOption::CONSERVATIVE ||
                value_arg.as_ref().unwrap().is_mutable() {
                    task_arg.cancel()
            }
        });

        task.add_listener(move |_| {
            complete_fn(&found_shadow.borrow().clone());
        });
        self.task_manager.add(task as Box<dyn Task>);
    }

    pub(crate) fn store_value<F>(&self, value: &Value, _: F)
    where F: Fn(&Option<Vec<Box<Node>>>) + 'static {
        let mut task = Box::new(NodeLookupTask::new(&value.id()));
        task.with_want_token(true);
        task.with_name("store_value");
        task.add_listener(move |task_arg| {
            if task_arg.state() != task::State::FINISHED {
                return;
            }

            /*
            auto closestSet = (static_cast<NodeLookup*>(t))->getClosestSet();
        if (closestSet.size() == 0) {
            // this should never happen
            log->warn("!!! Value announce task not started because the node lookup task got the empty closest nodes.");
            completeHandler({});
            return;
        }

        auto announce = std::make_shared<ValueAnnounce>(this, closestSet, value);
        announce->addListener([=](Task*) {
            std::list<Sp<NodeInfo>> result{};
            for(const auto& item: closestSet.getEntries()) {
                result.push_back(item);
            }
            completeHandler(result);
        });
        announce->setName("Nested value Store");
        t->setNestedTask(announce);
        taskMan.add(announce);*/

        });

        self.task_manager.add(task as Box<dyn Task>);
    }

    pub(crate) fn find_peer<F>(&self, id: &Id, expected: usize, option: LookupOption, complete_fn: F)
    where F: Fn(&Option<Vec<Box<Peer>>>) + 'static {
        let mut found: Option<Vec<Box<Peer>>> = None;
        let mut task = Box::new(PeerLookupTask::new(id));
        task.with_name("peer-lookup");

        let found_rc = Rc::new(RefCell::new(found.take()));
        let found_shadow = Rc::clone(&found_rc);

        task.set_result_fn(move |_, task_arg| {
            let found_borrowed = found_rc.borrow_mut();
            // peers->insert(peers->end(), listOfPeers.begin(), listOfPeers.end());
            if option != LookupOption::CONSERVATIVE && (*found_borrowed).as_ref().unwrap().len() >= expected {
                task_arg.cancel();
                return;
            }
        });

        task.add_listener(move |_| {
            complete_fn(&found_shadow.borrow())
        });
        self.task_manager.add(task as Box<dyn Task>);
    }

    pub(crate) fn announce_peer<F>(&self, _: &Peer, _: F)
    where F: Fn(&[&Node]) {
        unimplemented!()
    }

    pub(crate) fn on_timeout(&self, call: &RpcCall) {
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

    fn on_message<T>(&self, msg: &Box<T> )
    where T: Msg + lookup::Condition + find_value_req::ValueOption + store_value_req::StoreOption +
        announce_peer_req::AnnounceOption + error::ErrorResult{
        match msg.kind() {
            msg::Kind::Error => self.on_error(msg),
            msg::Kind::Request => self.on_request(msg),
            msg::Kind::Response => self.on_response(msg.as_ref()),
        }
    }

    fn on_request<T>(&self, msg: &Box<T>)
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

        self.rpcserver.send_msg(err);
    }


    fn on_ping(&self, request: &dyn Msg) {
        let mut msg = Box::new(ping_req::Message::new());

        msg.with_id(request.id());
        msg.with_txid(request.txid());
        msg.with_addr(request.addr());

        self.rpcserver.send_msg(msg);
    }

    fn on_find_node<T>(&self, request: &Box<T>) where T: Msg + lookup::Condition {
        let mut resp = Box::new(find_node_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        resp.populate_closest_nodes4(request.want4(), || {
            Some(KClosestNodes::new(
                self, // TODO: about DHT reference
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_closest_nodes6(request.want6(), || {
            Some(KClosestNodes::new(
                self, // TODO: about DHT reference
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_token(request.want_token(), || {
                self.token_manager.generate_token()
            }
        );

        self.rpcserver.send_msg(resp)
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
            Some(KClosestNodes::new(self,
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_closest_nodes6(request.want6() && has_value, || {
            Some(KClosestNodes::new(self,
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_token(request.want_token(), || {
            self.token_manager.generate_token()
        });

        self.rpcserver.send_msg(resp);
    }

    fn on_store_value<T>(&self, request: &Box<T>)
    where T: Msg + lookup::Condition + store_value_req::StoreOption {
        let value = request.value();
        let value_id = value.id();

        if !self.token_manager.verify_token(request.token(), request.id(), request.addr(), &value_id) {
            warn!("Received a store value request with invalid token from {}", request.addr());
            self.send_err(request.as_ref(), 203, "Invalid token for STORE VALUE request");
            return;
        }

        if value.is_valid().is_err() {
            self.send_err(request.as_ref(), 203, "Invalid value");
            return;
        }
        // TODO: store value.
        let mut resp = Box::new(store_value_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        self.rpcserver.send_msg(resp);
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
            Some(KClosestNodes::new(self,
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_closest_nodes6(request.want6() && has_peers, || {
            Some(KClosestNodes::new(self,
                request.target(),
                constants::MAX_ENTRIES_PER_BUCKET
            ).fill(true).as_nodes())
        });

        resp.populate_token(request.want_token(), || {
            self.token_manager.generate_token()
        });

        self.rpcserver.send_msg(resp);
    }

    fn on_announce_peer<T>(&self, request: &Box<T>)
    where T: Msg + lookup::Condition + announce_peer_req::AnnounceOption {
        let bogon = false;

        if bogon {
            info!("Received an announce peer request from bogon address {}, ignored ", request.addr());
        }

        if !self.token_manager.verify_token(request.token(), request.id(), request.addr(), request.target()) {
            warn!("Received an announce peer request with invalid token from {}", request.addr());
            self.send_err(request.as_ref(), 203, "Invalid token for ANNOUNCE PEER request");
            return;
        }

        let peers = request.peers();
        for peer in peers.iter() {
            if peer.is_valid().is_err() {
                self.send_err(request.as_ref(), 203, "One peer is invalid peer");
                return;
            }
        };

        debug!("Received an announce peer request from {}, saving peer {}", request.addr(), request.target());
        // TODO: Store peers.

        let mut resp = Box::new(announce_peer_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        self.rpcserver.send_msg(resp);
    }
}
