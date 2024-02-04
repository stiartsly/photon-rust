use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use log::{info, warn, debug};

use crate::{
    constants,
    version,
    id::Id,
    lookup_option::LookupOption,
    node::Node,
    peer::Peer,
    value::Value,
    rpccall::RpcCall,
    rpcserver::RpcServer,
    kclosest_nodes::KClosestNodes,
    token_man::TokenManager,
    routing_table::RoutingTable
};

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
    node_lookup::NodeLookupTask,
    value_lookup::ValueLookupTask,
    peer_lookup::PeerLookupTask
};

pub(crate) struct DHT {
    // node: Rc<NodeRunner>,

    server: Option<Rc<RefCell<RpcServer>>>,
    token_man: Option<Rc<RefCell<TokenManager>>>,

    addr: SocketAddr,

    routing_table: RoutingTable,
    task_man: TaskManager,

    persist_path: String,
    running: bool,
}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(binding_addr: &SocketAddr) -> Self {
        DHT {
            server: None,
            token_man: None,

            addr: binding_addr.clone(),

            routing_table: RoutingTable::new(),
            task_man: TaskManager::new(),
            running: false,

            persist_path: String::new(),
        }
    }

    pub(crate) fn set_server(&mut self, server: Rc<RefCell<RpcServer>>) {
        self.server = Some(server)
    }

    pub(crate) fn unset_server(&mut self) {
        _ = self.server.take()
    }

    fn server(&self) -> &Rc<RefCell<RpcServer>> {
        self.server.as_ref().unwrap()
    }

    pub(crate) fn set_token_manager(&mut self, token_man: Rc<RefCell<TokenManager>>) {
        self.token_man = Some(token_man)
    }

    pub(crate) fn unset_token_manager(&mut self) {
        _ = self.token_man.take()
    }

    fn token_man(&self) -> &Rc<RefCell<TokenManager>> {
        self.token_man.as_ref().unwrap()
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_path = path.to_string()
    }

    pub(crate) fn origin(&self) -> &SocketAddr {
        &self.addr
    }

    pub(crate)fn bootstrap() {
        unimplemented!()
    }

    pub(crate) fn find_node<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Box<Node>>) + 'static {
        let result = Rc::new(RefCell::new(
            self.routing_table.bucket_entry(id).map(
                |item| Box::new(item.node().clone())
            )
        ));
        let result_shadow = Rc::clone(&result);

        let mut task = Box::new(NodeLookupTask::new(id));
        task.with_name("node-lookup");
        task.set_result_fn(move|_task, _node| {
            if _node.is_some() {
                *(result.borrow_mut()) = Some(_node.unwrap());
            }
            if option == LookupOption::Conservative {
                _task.cancel()
            }
        });
        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        });

        self.task_man.add(task);
    }

    pub(crate) fn find_value<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Box<Value>>) + 'static {
        let result = Rc::new(RefCell::new(
            Option::default() as Option<Box<Value>>
        ));
        let result_shadow = Rc::clone(&result);

        let mut task = Box::new(ValueLookupTask::new(id));
        task.with_name("value-lookup");
        task.set_result_fn(move |_task, _value| {
            let binding = result.borrow();
            match binding.as_ref() {
                Some(v) => {
                    if let Some(_v) = _value.as_ref() {
                        if _v.is_mutable() && _v.sequence_number() < v.sequence_number() {
                            *(result.borrow_mut()) = Some(_v.clone());
                        }
                    }
                },
                None => {
                    if let Some(_v) = _value.as_ref() {
                        *(result.borrow_mut()) = Some(_v.clone())
                    }
                }
            }
            if option != LookupOption::Conservative {
                if let Some(_v) = _value {
                    if _v.is_mutable() {
                        _task.cancel()
                    }
                }
            }
        });

        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        });
        self.task_man.add(task);
    }

    pub(crate) fn store_value<F>(&self, _: &Value, _: F)
    where F: Fn(Option<Vec<Box<Node>>>) + 'static {
        unimplemented!()
    }

    pub(crate) fn find_peer<F>(&self, id: &Id, expected: usize, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Vec<Box<Peer>>>) + 'static {
        let result = Rc::new(RefCell::new(
            Option::default() as Option<Vec<Box<Peer>>>
        ));
        let result_shadow = Rc::clone(&result);

        let mut task = Box::new(PeerLookupTask::new(id));
        task.with_name("peer-lookup");
        task.set_result_fn(move |_task, _| {
            let found_borrowed = result.borrow_mut();
            // peers->insert(peers->end(), listOfPeers.begin(), listOfPeers.end());
            if option != LookupOption::Conservative && (*found_borrowed).as_ref().unwrap().len() >= expected {
                _task.cancel();
                return;
            }
        });

        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take())
        });

        self.task_man.add(task);
    }

    pub(crate) fn announce_peer<F>(&self, _: &Peer, _: F)
    where F: Fn(&[&Node]) {
        unimplemented!()
    }

    pub(crate) fn on_timeout(&self, call: &RpcCall) {
        // ignore the timeout if the DHT is stopped or the RPC server is offline
        if !self.running || !self.server().borrow().is_reachable() {
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
            version::formatted_version(msg.version()),
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

        self.server().borrow().send_msg(err);
    }


    fn on_ping(&self, request: &dyn Msg) {
        let mut msg = Box::new(ping_req::Message::new());

        msg.with_id(request.id());
        msg.with_txid(request.txid());
        msg.with_addr(request.addr());

        self.server().borrow().send_msg(msg);
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
                self.token_man().borrow().generate_token(
                    request.id(),
                    request.addr(),
                    request.target()
                )
            }
        );

        self.server().borrow().send_msg(resp)
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
            self.token_man().borrow().generate_token(
                request.id(),
                request.addr(),
                request.target()
            )
        });

        self.server().borrow().send_msg(resp);
    }

    fn on_store_value<T>(&mut self, request: &Box<T>)
    where T: Msg + lookup::Condition + store_value_req::StoreOption {
        let value = request.value();
        let value_id = value.id();

        if !self.token_man().borrow_mut().verify_token(request.token(), request.id(), request.addr(), &value_id) {
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

        self.server().borrow().send_msg(resp);
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
            self.token_man().borrow().generate_token(
                request.id(),
                request.addr(),
                request.target()
            )
        });

        self.server().borrow().send_msg(resp);
    }

    fn on_announce_peer<T>(&mut self, request: &Box<T>)
    where T: Msg + lookup::Condition + announce_peer_req::AnnounceOption {
        let bogon = false;

        if bogon {
            info!("Received an announce peer request from bogon address {}, ignored ",
                request.addr()
            );
        }

        if !self.token_man().borrow_mut().verify_token(
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

        self.server().borrow().send_msg(resp);
    }
}
