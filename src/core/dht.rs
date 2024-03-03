
use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use log::{debug, info, warn};

use crate::{
    constants,
    version,
    id::Id,
    node_info::NodeInfo,
    peer::Peer,
    value::Value,
    rpccall::RpcCall,
    lookup_option::LookupOption,
    routing_table::RoutingTable,
    kclosest_nodes::KClosestNodes,
    token_man::TokenManager,
    server::Server,
    scheduler::Scheduler,
};

use crate::msg::{
    ping_req,
    announce_peer_req,
    announce_peer_rsp,
    find_node_rsp,
    find_value_req,
    store_value_req,
    store_value_rsp,
    error::{self, ErrorResult},
    find_peer_rsp::{self, PeerResult},
    find_value_rsp::{self, ValueResult},
    lookup::{self, Result},
    msg::{self, Msg},
};

use crate::task::{
    task::{State, Task},
    lookup::LookupTask,
    node_lookup::NodeLookupTask,
    peer_lookup::PeerLookupTask,
    task_manager::TaskManager,
    value_lookup::ValueLookupTask,
};

pub(crate) struct DHT {
    addr: SocketAddr,
    persist_path: String,
    running: bool,

    routing_table: RoutingTable,

    server:     Rc<RefCell<Server>>,
    task_man:   Rc<RefCell<TaskManager>>,
    token_man:  Rc<RefCell<TokenManager>>,
    scheduler:  Rc<RefCell<Scheduler>>,
}

const DHT_UPDATE_INTERVAL: u64 = 1000; // 1 seconds;
const RANDOM_PING_INTERVAL: u64 = 10 * 1000; // 10 seconds;
const RANDOM_LOOKUP_INTERVAL: u64 = 10 * 60 * 1000; // 10 minutes;

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(server: &Rc<RefCell<Server>>, binding_addr: SocketAddr) -> Self {
        DHT {
            addr: binding_addr,
            running: false,
            persist_path: String::new(),
            routing_table: RoutingTable::new(),

            server: Rc::clone(server),
            task_man:  Rc::new(RefCell::new(TaskManager::new())),
            token_man: Rc::clone(server.borrow().token_man()),
            scheduler: Rc::clone(server.borrow().scheduler()),
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_path.push_str(path);
    }

    pub(crate) fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub(crate) fn is_ipv4(&self) -> bool {
        self.addr.is_ipv4()
    }

    fn bootstrap_internal() {
        unimplemented!()
    }

    pub(crate) fn bootstrap(&mut self, _: &[NodeInfo]) {
        unimplemented!()
    }

    fn fill_home_bucket(&mut self, _: &[NodeInfo]) {
        unimplemented!()
    }

    fn update(&mut self) {
        unimplemented!()
    }

    pub(crate) fn start(&mut self) {
        if self.is_running() {
            return;
        }

        // Load neighboring nodes from cache storage if possible.
        let path = self.persist_path.as_str();
        if !path.is_empty() {
            info!("Loading routing table from [{}] ...", path);
            self.routing_table.load(path);
        }

        // TODO: bootstrap nodes.

        info!("Starting DHT/{} on {}", addr_kind(&self.addr), self.addr);
        self.running = true;

        // Maintenance tasks should run continuously, even before the first queries
        // are executed.
        let task_man = Rc::clone(&self.task_man);
        self.scheduler.borrow_mut().add(
            500,
            DHT_UPDATE_INTERVAL,
            move || {
                task_man.borrow_mut().dequeue();
            }
        );

        // fix the first time to persist the routing table: 2 min
        //lastSave = currentTimeMillis() - Constants::ROUTING_TABLE_PERSIST_INTERVAL + (120 * 1000);

        // Regularly DHT update
        self.scheduler.borrow_mut().add(
            100,
            DHT_UPDATE_INTERVAL,
            move || {
                // TODO;
            }
        );

        // Send a ping request to a random node to verify socket liveness.
        self.scheduler.borrow_mut().add(
            RANDOM_PING_INTERVAL,
            RANDOM_PING_INTERVAL,
            move || {
                // TODO;
            }
        );

        // Perform a deep lookup to familiarize ourselves with random sections of
        // the keyspace.
        let mut kind = String::from(addr_kind(&self.addr));
        let task_man = Rc::clone(&self.task_man);
        self.scheduler.borrow_mut().add(
            RANDOM_LOOKUP_INTERVAL,
            RANDOM_LOOKUP_INTERVAL,
            move || {
                let mut task = Box::new(NodeLookupTask::new(&Id::random()));
                kind.push_str(":Random refresh lookup");
                task.with_name(&kind);
                task.add_listener(move |_|{});

                task_man.borrow_mut().add(task);
            }
        )
    }

    pub(crate) fn stop(&mut self) {
        if !self.is_running() {
            return;
        }

        info!("{} initiated shutdown ...", addr_kind(&self.addr));
        info!("stopping servers ...");

        self.running = false;

        info!("Persisting routing table on shutdown ...");
        self.routing_table.save(self.persist_path.as_str());

        self.task_man.borrow_mut().cancel_all();
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running
    }

    fn on_message<T>(&mut self, msg: &Box<T>)
    where
        T: Msg
            + lookup::Condition
            + find_value_req::ValueOption
            + store_value_req::StoreOption
            + announce_peer_req::AnnounceOption
            + error::ErrorResult,
    {
        match msg.kind() {
            msg::Kind::Error => self.on_error(msg),
            msg::Kind::Request => self.on_request(msg),
            msg::Kind::Response => self.on_response(msg.as_ref()),
        }
    }

    fn on_request<T>(&mut self, msg: &Box<T>)
    where
        T: Msg
            + lookup::Condition
            + find_value_req::ValueOption
            + store_value_req::StoreOption
            + announce_peer_req::AnnounceOption,
    {
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

    fn on_error<T>(&self, msg: &Box<T>)
    where
        T: Msg + error::ErrorResult,
    {
        warn!(
            "Error from {}/{} - {}:{}, txid {}",
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
        err.with_ver(version::build(version::NODE_TAG_NAME, version::NODE_VERSION));
        err.with_txid(msg.txid());
        err.with_addr(msg.addr());
        err.with_msg(str);
        err.with_code(code);

        self.server.borrow_mut().send_msg(err, self.is_ipv4());
    }

    fn on_ping(&self, request: &dyn Msg) {
        let mut msg = Box::new(ping_req::Message::new());

        msg.with_id(request.id());
        msg.with_txid(request.txid());
        msg.with_addr(request.addr());

        self.server.borrow_mut().send_msg(msg, self.is_ipv4());
    }

    fn on_find_node<T>(&self, request: &Box<T>)
    where
        T: Msg + lookup::Condition,
    {
        let mut resp = Box::new(find_node_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        resp.populate_closest_nodes4(request.want4(), || {
            Some(
                KClosestNodes::new(
                    Rc::new(self),
                    request.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes(),
            )
        });

        resp.populate_closest_nodes6(request.want6(), || {
            Some(
                KClosestNodes::new(
                    Rc::new(self),
                    request.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes(),
            )
        });

        resp.populate_token(request.want_token(), || {
            self.token_man
                .borrow()
                .generate_token(request.id(), request.addr(), request.target())
        });

        self.server.borrow_mut().send_msg(resp, self.is_ipv4())
    }

    fn on_find_value<T>(&self, request: &Box<T>)
    where
        T: Msg + lookup::Condition + find_value_req::ValueOption,
    {
        let mut resp = Box::new(find_value_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        let mut has_value = false;
        resp.populate_value(|| {
            // TODO;
            let value: Option<Box<Value>> = None;
            if value.is_some() {
                if request.seq() < 0
                    || value.as_ref().unwrap().sequence_number() < 0
                    || request.seq() <= value.as_ref().unwrap().sequence_number()
                {
                    has_value = true;
                }
            }
            value
        });

        resp.populate_closest_nodes4(request.want4() && has_value, || {
            Some(
                KClosestNodes::new(
                    Rc::new(&self),
                    request.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes(),
            )
        });

        resp.populate_closest_nodes6(request.want6() && has_value, || {
            Some(
                KClosestNodes::new(
                    Rc::new(&self),
                    request.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes(),
            )
        });

        resp.populate_token(request.want_token(), || {
            self.token_man
                .borrow()
                .generate_token(request.id(), request.addr(), request.target())
        });

        self.server.borrow_mut().send_msg(resp, self.is_ipv4());
    }

    fn on_store_value<T>(&mut self, request: &Box<T>)
    where
        T: Msg + lookup::Condition + store_value_req::StoreOption,
    {
        let value = request.value();
        let value_id = value.id();

        if !self.token_man.borrow_mut().verify_token(
            request.token(),
            request.id(),
            request.addr(),
            &value_id,
        ) {
            warn!(
                "Received a store value request with invalid token from {}",
                request.addr()
            );
            self.send_err(
                request.as_ref(),
                203,
                "Invalid token for STORE VALUE request",
            );
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

        self.server.borrow_mut().send_msg(resp, self.is_ipv4());
    }

    fn on_find_peers<T>(&self, request: &Box<T>)
    where
        T: Msg + lookup::Condition + find_value_req::ValueOption,
    {
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
            Some(
                KClosestNodes::new(
                    Rc::new(&self),
                    request.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes(),
            )
        });

        resp.populate_closest_nodes6(request.want6() && has_peers, || {
            Some(
                KClosestNodes::new(
                    Rc::new(&self),
                    request.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes(),
            )
        });

        resp.populate_token(request.want_token(), || {
            self.token_man
                .borrow()
                .generate_token(request.id(), request.addr(), request.target())
        });

        self.server.borrow_mut().send_msg(resp, self.is_ipv4());
    }

    fn on_announce_peer<T>(&mut self, request: &Box<T>)
    where
        T: Msg + lookup::Condition + announce_peer_req::AnnounceOption,
    {
        let bogon = false;

        if bogon {
            info!(
                "Received an announce peer request from bogon address {}, ignored ",
                request.addr()
            );
        }

        if !self.token_man.borrow_mut().verify_token(
            request.token(),
            request.id(),
            request.addr(),
            request.target(),
        ) {
            warn!(
                "Received an announce peer request with invalid token from {}",
                request.addr()
            );
            self.send_err(
                request.as_ref(),
                203,
                "Invalid token for ANNOUNCE PEER request",
            );
            return;
        }

        let peers = request.peers();
        for peer in peers.iter() {
            if !peer.is_valid() {
                self.send_err(request.as_ref(), 203, "One peer is invalid peer");
                return;
            }
        }

        debug!(
            "Received an announce peer request from {}, saving peer {}",
            request.addr(),
            request.target()
        );
        // TODO: Store peers.

        let mut resp = Box::new(announce_peer_rsp::Message::new());

        resp.with_id(request.id());
        resp.with_txid(request.txid());
        resp.with_addr(request.addr());

        self.server.borrow_mut().send_msg(resp, self.is_ipv4());
    }

    pub(crate) fn on_timeout(&self, call: &RpcCall) {
        // ignore the timeout if the DHT is stopped or the RPC server is offline
        if !self.running || !self.server.borrow().is_reachable() {
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

    pub(crate) fn find_node<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where
        F: Fn(Option<Box<NodeInfo>>) + 'static,
    {
        let result = Rc::new(RefCell::new(
            self.routing_table
                .bucket_entry(id)
                .map(|item| Box::new(item.node().clone())),
        ));
        let result_shadow = Rc::clone(&result);

        let mut task = Box::new(NodeLookupTask::new(id));
        task.with_name("node-lookup");
        task.set_result_fn(move |_task, _node| {
            if _node.is_some() {
                *(result.borrow_mut()) = Some(_node.unwrap().clone());
            }
            if option == LookupOption::Conservative {
                _task.cancel()
            }
        });
        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        });

        self.task_man.borrow_mut().add(task);
    }

    pub(crate) fn find_value<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where
        F: Fn(Option<Box<Value>>) + 'static,
    {
        let result = Rc::new(RefCell::new(Option::default() as Option<Box<Value>>));
        let result_shadow = Rc::clone(&result);

        let mut task = Box::new(ValueLookupTask::new(id));
        task.with_name("value-lookup");
        task.set_result_fn(move |_task, _value| {
            if let Some(_v) = _value.as_ref() {
                match result.borrow().as_ref() {
                    Some(v) => {
                        if _v.is_mutable() && v.sequence_number() < _v.sequence_number() {
                            *(result.borrow_mut()) = Some(_v.clone());
                        }
                    }
                    None => *(result.borrow_mut()) = Some(_v.clone()),
                }
            }
            if option != LookupOption::Conservative {
                if let Some(_v) = _value {
                    if !_v.is_mutable() {
                        _task.cancel()
                    }
                }
            }
        });

        task.add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        });
        self.task_man.borrow_mut().add(task);
    }

    pub(crate) fn store_value<F>(&self, value: &Value, complete_fn: F)
    where
        F: Fn(Option<Vec<Box<NodeInfo>>>) + 'static,
    {
        let mut task = Box::new(NodeLookupTask::new(&value.id()));
        task.with_name("store-value");
        task.set_want_token(true);
        task.add_listener(move |_task| {
            if _task.state() != State::Finished {
                return;
            }

            if let Some(downcasted) = _task.as_any().downcast_ref::<NodeLookupTask>() {
                let closest_set = downcasted.closest_set();
                if closest_set.size() == 0 {
                    // This hould never happen
                    warn!("!!! Value announce task not started because the node lookup task got the empty closest nodes.");
                    complete_fn(Option::default());
                    return;
                }
                // TODO:
            }
        });
        self.task_man.borrow_mut().add(task);
    }

    pub(crate) fn find_peer<F>(
        &self,
        id: &Id,
        expected: usize,
        option: LookupOption,
        complete_fn: F,
    ) where
        F: Fn(Vec<Box<Peer>>) + 'static,
    {
        let result = Rc::new(RefCell::new(Vec::new() as Vec<Box<Peer>>));
        let result_shadow = Rc::clone(&result);

        let mut task = Box::new(PeerLookupTask::new(id));
        task.with_name("peer-lookup");
        task.set_result_fn(move |_task, _peers| {
            (*result.borrow_mut()).append(_peers);
            if option != LookupOption::Conservative && result.borrow().len() >= expected {
                _task.cancel()
            }
        });

        task.add_listener(move |_| complete_fn(result_shadow.take()));

        self.task_man.borrow_mut().add(task);
    }

    pub(crate) fn announce_peer<F>(&self, _: &Peer, _: F)
    where
        F: Fn(&[&NodeInfo]),
    {
        unimplemented!()
    }
}

fn addr_kind(addr: &SocketAddr) -> &'static str {
    match addr.is_ipv4() {
        true => "Ipv4",
        false => "ipv6"
    }
}
