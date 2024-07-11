
use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;
use std::net::SocketAddr;
use std::ops::Deref;
use log::{debug, info, warn, trace};

use crate::{
    unwrap,
    is_bogon_addr,
    as_kind_name,
    as_millis,
    constants,
    version,
    id::Id,
    node_info::NodeInfo,
    peer::Peer,
    value::Value,
    rpccall::{self, RpcCall},
    lookup_option::LookupOption,
    routing_table::RoutingTable,
    kclosest_nodes::KClosestNodes,
    token_man::TokenManager,
    server::Server,
    kbucket_entry::KBucketEntry,
    data_storage::DataStorage,
};

use crate::msg::{
    lookup_req::{Msg as LookupRequest},
    lookup_rsp::{Msg as LookupResponse},
    ping_req,
    ping_rsp,
    find_node_req,
    find_node_rsp,
    find_peer_req,
    find_peer_rsp,
    store_value_req,
    store_value_rsp,
    find_value_req,
    find_value_rsp,
    announce_peer_req,
    announce_peer_rsp,
    error,
    msg::{Msg, Kind, Method},
};

use crate::task::{
    task::{State, Task},
    lookup_task::LookupTask,
    node_lookup::NodeLookupTask,
    peer_lookup::PeerLookupTask,
    task_manager::TaskManager,
    value_lookup::ValueLookupTask,
};

pub(crate) struct DHT {
    nodeid: Id,
    addr: SocketAddr,
    persist_path: Option<String>,
    last_save: SystemTime,
    running: bool,

    bootstrap_need: bool,
    bootstrap_nodes: Vec<Box<NodeInfo>>,
    bootstrap_time: Rc<RefCell<SystemTime>>,

    routing_table: Rc<RefCell<RoutingTable>>,
    taskman: Rc<RefCell<TaskManager>>,

    server: Option<Rc<RefCell<Server>>>,
    tokenman: Option<Rc<RefCell<TokenManager>>>,
    storage: Option<Rc<RefCell<dyn DataStorage>>>,
    cloned_dht: Option<Rc<RefCell<DHT>>>,
}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(nodeid: &Id, binding_addr: &SocketAddr) -> Self {
        let node_info = NodeInfo::new(nodeid, &binding_addr);

        DHT {
            nodeid: nodeid.clone(),
            addr: binding_addr.clone(),
            running: false,
            persist_path: None,
            last_save: SystemTime::UNIX_EPOCH,

            routing_table: Rc::new(RefCell::new(RoutingTable::new(node_info))),
            taskman:  Rc::new(RefCell::new(TaskManager::new())),

            bootstrap_nodes: Vec::new(),
            bootstrap_need: false,
            bootstrap_time: Rc::new(RefCell::new(SystemTime::UNIX_EPOCH)),

            server: None,
            storage: None,
            tokenman: None,
            cloned_dht: None,
        }
    }

    pub(crate) fn set_cloned(&mut self, dht: &Rc<RefCell<DHT>>) {
        self.cloned_dht = Some(Rc::clone(dht));
    }

    pub(crate) fn set_server(&mut self, server: &Rc<RefCell<Server>>) {
        self.server = Some(Rc::clone(server));
    }

    pub(crate) fn set_storage(&mut self, storage: &Rc<RefCell<dyn DataStorage>>) {
        self.storage = Some(Rc::clone(storage));
    }

    pub(crate) fn set_tokenman(&mut self, tokenman: &Rc<RefCell<TokenManager>>) {
        self.tokenman = Some(Rc::clone(tokenman));
    }

    pub(crate) fn cloned(&self) -> Rc<RefCell<DHT>> {
        Rc::clone(unwrap!(self.cloned_dht))
    }

    pub(crate) fn enable_persistence(&mut self, path: String) {
        self.persist_path = Some(path);
    }

    pub(crate) fn add_bootstrap_node(&mut self, node: Box<NodeInfo>) {
        self.bootstrap_nodes.push(node);
        self.bootstrap_need = true;
    }

    pub(crate) fn socket_addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub(crate) fn node_id(&self) -> &Id {
        &self.nodeid
    }

    pub(crate) fn routing_table(&self) -> Rc<RefCell<RoutingTable>> {
        Rc::clone(&self.routing_table)
    }

    pub(crate) fn server(&self) -> Rc<RefCell<Server>> {
        Rc::clone(self.server.as_ref().unwrap())
    }

    fn storage(&self) -> &Rc<RefCell<dyn DataStorage>> {
        self.storage.as_ref().unwrap()
    }

    fn tokenman(&self) -> &Rc<RefCell<TokenManager>> {
        self.tokenman.as_ref().unwrap()
    }

    pub(crate) fn bootstrap(&mut self) {
        let bns = match self.bootstrap_nodes.is_empty() {
            true => self.routing_table.borrow().random_entries(8).unwrap(),
            false => self.bootstrap_nodes.clone()
        };

        info!("DHT/{} bootstraping ....", as_kind_name!(&self.addr));

        let nodes = Rc::new(RefCell::new(Vec::new())) as Rc<RefCell<Vec<NodeInfo>>>;
        let count = Rc::new(RefCell::new(0));

        for node in bns.iter() {
            let req = Rc::new(RefCell::new(find_node_req::Message::new()));
            req.borrow_mut().set_remote(node.id(), node.socket_addr());
            req.borrow_mut().with_target(Id::random());
            req.borrow_mut().with_want4(true);

            let call = Rc::new(RefCell::new(RpcCall::new(node.clone(), req)));
            let len = bns.len();
            let cloned_nodes = Rc::clone(&nodes);
            let cloned_count = Rc::clone(&count);
            let cloned_id = Rc::new(node.id().clone());
            let taskman = Rc::clone(&self.taskman);
            let routing_table = Rc::clone(&self.routing_table);
            let bootstrap_time = Rc::clone(&self.bootstrap_time);
            let cloned_dht = self.cloned();
            call.borrow_mut().set_state_changed_fn(move |call, _, cur| {
                if cur == &rpccall::State::Responsed || cur == &rpccall::State::Err ||
                    cur == &rpccall::State::Timeout {
                    if let Some(rsp) = call.rsp() {
                        if let Some(downcasted) = rsp.borrow().as_any().downcast_ref::<find_node_rsp::Message>() {
                            cloned_nodes.borrow_mut().extend_from_slice(downcasted.nodes4().unwrap());
                        }
                    }

                    *cloned_count.borrow_mut() += 1;
                    if *cloned_count.borrow() == len {

                        *bootstrap_time.borrow_mut() = SystemTime::now();
                        if routing_table.borrow().size() == 0 &&
                            cloned_nodes.borrow().is_empty() {
                            return;
                        }


                        let task = Rc::new(RefCell::new(NodeLookupTask::new(
                            cloned_id.deref(), Rc::clone(&cloned_dht)
                        )));
                        let cloned_task = Rc::clone(&task);
                        task.borrow_mut().cloned_self(cloned_task);
                        task.borrow_mut().set_bootstrap(true);
                        task.borrow_mut().inject_candidates(cloned_nodes.borrow().as_slice());
                        task.borrow_mut().set_name("Bootstrap: filling home bucket");
                        task.borrow_mut().add_listener(Box::new(move |_| {
                             println!(">>>>>> listener invoked!!!! >>>>");
                        }));
                        taskman.borrow_mut().add(task);
                    }
                }
            });

            self.server().borrow_mut().send_call(call);
        };
    }

    pub(crate) fn update(&mut self) {
        if !self.running {
            return;
        }

        trace!("DHT/{} regularly update...", as_kind_name!(&self.addr));
        self.server().borrow_mut().update_reachability();
        self.routing_table.borrow_mut().maintenance();

        if self.bootstrap_need || self.routing_table.borrow().size() < constants::BOOTSTRAP_IF_LESS_THAN_X_PEERS ||
            as_millis!(self.bootstrap_time.borrow()) > constants::SELF_LOOKUP_INTERVAL {

            self.bootstrap_need = false;
            // Regularly search for our ID to update the routing table
            self.bootstrap();
        }

        if as_millis!(self.last_save) > constants::ROUTING_TABLE_PERSIST_INTERVAL {
            info!("Persisting routing table ....");
            self.routing_table.borrow_mut().save(self.persist_path.as_ref().unwrap().as_str());
            self.last_save = SystemTime::now();
        }
    }

    pub(crate) fn random_ping(&mut self) {
        println!(">>>>> random_ping >>>>>");

        if self.server().borrow().number_of_acitve_calls() > 0 {
            return;
        }

        let binding = self.routing_table.borrow();
        let entry = match binding.random_entry() {
            Some(entry) => entry,
            None => return,
        };

        let req = Rc::new(RefCell::new(ping_req::Message::new()));
        let call = Rc::new(RefCell::new(RpcCall::new(entry.clone(), req)));
        self.server().borrow_mut().send_call(call);
    }

    pub(crate) fn random_lookup(&mut self) {
        let mut task = NodeLookupTask::new(&Id::random(), self.cloned());
        let name = format!("{}: random lookup", as_kind_name!(&self.addr));
        task.set_name(&name);
        task.add_listener(Box::new(move |_|{}));

        let task = Rc::new(RefCell::new(task));
        let task_cloned = Rc::clone(&task);
        task.borrow_mut().cloned_self(task_cloned);
        self.taskman.borrow_mut().add(task);
    }

    pub(crate) fn persist_announce(&self) {
        info!("Reannounce the perisitent values and peers...");
        // TODO:
    }

    pub(crate) fn start(&mut self) {
        if self.running {
            return;
        }

        // Load neighboring nodes from cache storage if they exist.
        if let Some(path) = self.persist_path.as_ref() {
            info!("Loading routing table from [{}] ...", path);
            self.routing_table.borrow_mut().load(path);
        }

        // TODO: bootstrap nodes.

        info!("Starting DHT/{} on {}", as_kind_name!(&self.addr), self.addr);
        self.running = true;

        // Task management.
        let scheduler = self.server().borrow().scheduler();
        let taskman = Rc::clone(&self.taskman);
        scheduler.borrow_mut().add(move || {
            taskman.borrow_mut().dequeue();
        }, 500, constants::DHT_UPDATE_INTERVAL);

        // fix the first time to persist the routing table: 2 min
        //lastSave = currentTimeMillis() - Constants::ROUTING_TABLE_PERSIST_INTERVAL + (120 * 1000);

        // Regular dht update.
        let cloned_dht = self.cloned();
        scheduler.borrow_mut().add(move || {
            cloned_dht.borrow_mut().update();
        }, 100, constants::DHT_UPDATE_INTERVAL);

        // Send a ping request to a random node to verify socket liveness.
        let cloned_dht = self.cloned();
        scheduler.borrow_mut().add(move || {
            cloned_dht.borrow_mut().random_ping();
        }, constants::RANDOM_PING_INTERVAL, constants::RANDOM_PING_INTERVAL);

        // Perform a deep lookup to familiarize ourselves with random sections of
        // the keyspace.
        let cloned_dht = self.cloned();
        scheduler.borrow_mut().add(move || {
            cloned_dht.borrow_mut().random_lookup();
        }, constants::RANDOM_LOOKUP_INTERVAL, constants::RANDOM_LOOKUP_INTERVAL);

        let cloned_dht = self.cloned();
        scheduler.borrow_mut().add(move || {
            cloned_dht.borrow().persist_announce();
        }, 1000, constants::RE_ANNOUNCE_INTERVAL);
    }

    pub(crate) fn stop(&mut self) {
        if !self.running {
            return;
        }

        info!("{} initiated shutdown ...", as_kind_name!(&self.addr));
        info!("stopping servers ...");

        self.cloned_dht = None;
        self.running = false;

        info!("Persisting routing table on shutdown ...");
        if let Some(path) = self.persist_path.as_ref() {
            self.routing_table.borrow_mut().save(path);
        }
        self.taskman.borrow_mut().cancel_all();
    }

    pub(crate) fn on_message(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        let kind = msg.borrow().kind();
        let cloned = Rc::clone(&msg);
        match kind {
            Kind::Error => self.on_error(cloned),
            Kind::Request => self.on_request(cloned),
            Kind::Response => self.on_response(cloned),
        };
        self.received(msg);
    }

    fn received(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        let msg = msg.borrow();
        let from_id = msg.id();
        let from_addr = msg.origin();

        if is_bogon_addr!(from_addr) {
            info!("Received a message from bogon address {}, ignored the potential
                  routing table operation", from_addr);
            return;
        }

        let call = msg.associated_call();
        if let Some(call) = call.as_ref() {
            // we only want remote nodes with stable ports in our routing table,
            // so apply a stricter check here
            if !call.borrow().matches_addr() {
                return;
            }
        }

        let mut entry_found = false;
        if let Some(old) = self.routing_table.borrow().bucket_entry(from_id) {
            // this might happen if one node changes ports (broken NAT?) or IP address
            // ignore until routing table entry times out
            if old.node_addr() != self.socket_addr() {
                return;
            }
            entry_found = true;
        }

        let mut new_entry = Box::new(KBucketEntry::new(msg.id(), from_addr));
        if let Some(call) = call {
            new_entry.set_version(msg.ver());
            new_entry.signal_response();
            new_entry.merge_request_time(call.borrow().sent_time().clone());
        } else if !entry_found {
            let req = Rc::new(RefCell::new(ping_req::Message::new()));
            let ni = Box::new(new_entry.inner_node());
            let call = Rc::new(RefCell::new(RpcCall::new(ni, req)));
            self.server().borrow_mut().send_call(call);
        }
        self.routing_table.borrow_mut().put(new_entry);
    }

    fn on_request(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        let binding = msg.borrow();
        let msg = binding.deref();
        match msg.method() {
            Method::Ping => self.on_ping(msg),
            Method::FindNode => self.on_find_node(msg),
            Method::FindValue => self.on_find_value(msg),
            Method::StoreValue => self.on_store_value(msg),
            Method::FindPeer => self.on_find_peers(msg),
            Method::AnnouncePeer => self.on_announce_peer(msg),
            Method::Unknown => {
                self.send_err(msg, 203, "Invalid request method");
            }
        }
    }

    fn on_response(&mut self, _: Rc<RefCell<dyn Msg>>) {}

    fn on_error(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        let binding = msg.borrow();
        let req = binding.as_any().downcast_ref::<error::Message>().unwrap();
        warn!("Error from {}/{} - {}:{}, txid {}",
            req.origin(),
            version::formatted_version(binding.ver()),
            req.code(),
            req.msg(),
            req.txid()
        );
    }

    fn send_err(&mut self, msg: &dyn Msg, code: i32, str: &str) {
        let mut err = error::Message::with_txid(msg.method(), msg.txid());

        err.set_remote(msg.id(), msg.origin());
        err.set_ver(version::build(version::NODE_TAG_NAME, version::NODE_VERSION));
        err.set_txid(msg.txid());
        err.with_msg(str);
        err.with_code(code);

        self.server().borrow_mut().send_msg(
            Rc::new(RefCell::new(err))
        );
    }

    fn on_ping(&mut self, req: &dyn Msg) {
        let mut rsp = ping_rsp::Message::with_txid(req.txid());
        rsp.set_remote(req.id(), req.origin());

        self.server().borrow_mut().send_msg(
            Rc::new(RefCell::new(rsp))
        );
    }

    fn on_find_node(&mut self, msg: &dyn Msg) {
        let req = msg.as_any().downcast_ref::<find_node_req::Message>().unwrap();
        let mut rsp = find_node_rsp::Message::new();
        rsp.set_remote(req.id(), req.origin());
        rsp.set_txid(req.txid());

        if req.want4() {
            let mut knodes = KClosestNodes::new(
                req.target(),
                self.routing_table(),
                constants::MAX_ENTRIES_PER_BUCKET,
            );
            knodes.fill(true);
            rsp.populate_closest_nodes4(knodes.as_nodes());
        }

        if req.want_token() {
            let token = self.tokenman().borrow_mut().generate_token(
                req.id(), req.origin(), req.target()
            );
            rsp.populate_token(token);
        }

        self.server().borrow_mut().send_msg(
            Rc::new(RefCell::new(rsp))
        );
    }

    fn on_find_value(&mut self, msg: &dyn Msg) {
        let req = msg.as_any().downcast_ref::<find_value_req::Message>().unwrap();
        let mut rsp = find_value_rsp::Message::new();
        rsp.set_remote(req.id(), req.origin());
        rsp.set_txid(req.txid());

        let mut has_value = false;
        let value = self.storage().borrow().value(req.target());
        if value.is_some() {
            if req.seq() < 0
                || value.as_ref().unwrap().sequence_number() < 0
                || req.seq() <= value.as_ref().unwrap().sequence_number()
            {
                has_value = true;
                rsp.populate_value(value.unwrap());
            }
        }

        if req.want4() && has_value {
            let mut knodes = KClosestNodes::new(
                req.target(),
                self.routing_table(),
                constants::MAX_ENTRIES_PER_BUCKET,
            );
            knodes.fill(true);
            rsp.populate_closest_nodes4(knodes.as_nodes());
        }

        if req.want_token() {
            let token = self.tokenman().borrow_mut().generate_token(
                req.id(), req.origin(), req.target()
            );
            rsp.populate_token(token);
        }

        self.server().borrow_mut().send_msg(
            Rc::new(RefCell::new(rsp))
        );
    }

    fn on_store_value(&mut self, msg: &dyn Msg) {
        let req = msg.as_any().downcast_ref::<store_value_req::Message>().unwrap();
        let value = req.value();
        let value_id = value.as_ref().unwrap().id();

        let valid = self.tokenman().borrow_mut().verify_token(
            req.token(), req.id(), req.origin(), &value_id,
        );
        if !valid {
            warn!("Received a store value request with invalid token from {}", req.origin());
            self.send_err(msg, 203, "Invalid token for store value request");
            return;
        }

        if !value.as_ref().unwrap().is_valid() {
            self.send_err(msg, 203, "Invalid value");
            return;
        }

        // TODO: store value.
        let mut rsp = store_value_rsp::Message::new();
        rsp.set_remote(req.id(), req.origin());
        rsp.set_txid(req.txid());

        self.server().borrow_mut().send_msg(
            Rc::new(RefCell::new(rsp))
        );
    }

    fn on_find_peers(&mut self, msg: &dyn Msg) {
        let req = msg.as_any().downcast_ref::<find_peer_req::Message>().unwrap();
        let mut rsp = find_peer_rsp::Message::new();
        rsp.set_remote(req.id(), req.origin());
        rsp.set_txid(req.txid());

        let mut has_peers = false;
        let peers = self.storage().borrow().peers(req.target(), 8);
        if !peers.is_empty() {
            has_peers = true;
            rsp.populate_peers(peers);
        }

        if req.want4() && has_peers {
            let mut knodes = KClosestNodes::new(
                req.target(),
                self.routing_table(),
                constants::MAX_ENTRIES_PER_BUCKET,
            );
            knodes.fill(true);
            rsp.populate_closest_nodes4(knodes.as_nodes());
        }

        if req.want_token() {
            let token = self.tokenman().borrow_mut().generate_token(
                req.id(), req.origin(), req.target()
            );
            rsp.populate_token(token);
        }

        self.server().borrow_mut().send_msg(
            Rc::new(RefCell::new(rsp))
        );
    }

    fn on_announce_peer(&mut self, msg: &dyn Msg) {
        let req = msg.as_any().downcast_ref::<announce_peer_req::Message>().unwrap();
        if is_bogon_addr!(req.origin()) {
            info!("Received an announce peer request from bogon address {}, ignored ",
                req.origin()
            );
        }

        let valid = self.tokenman().borrow_mut().verify_token(
            req.token(), req.id(), req.origin(), req.target()
        );
        if !valid {
            warn!("Received an announce peer request with invalid token from {}", req.origin());
            self.send_err(msg, 203,"Invalid token for ANNOUNCE PEER request");
            return;
        }

        let peer = req.peer();
        if !peer.is_valid() {
            self.send_err(msg, 203, "One peer is invalid peer");
            return;
        }

        debug!( "Received an announce peer request from {}, saving peer {}",
            req.origin(), req.target());
        // TODO: Store peers.

        let mut rsp = announce_peer_rsp::Message::new();
        rsp.set_remote(req.id(), req.origin());
        rsp.set_txid(req.txid());

        self.server().borrow_mut().send_msg(
            Rc::new(RefCell::new(rsp))
        );
    }

    pub(crate) fn on_timeout(&mut self, call: &RpcCall) {
        // Ignore the timeout if the DHT is stopped or the RPC server is offline
        if !self.running || !self.server().borrow().is_reachable() {
            return;
        }
        self.routing_table.borrow_mut().on_timeout(call.target_nodeid());
    }

    pub(crate) fn on_send(&mut self, id: &Id) {
        if !self.running {
            return;
        }
        self.routing_table.borrow_mut().on_send(id)
    }

    pub(crate) fn find_node<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Box<NodeInfo>>) + 'static
    {
        let result = Rc::new(RefCell::new(
            self.routing_table.borrow()
                .bucket_entry(id)
                .map(|item| Box::new(item.inner_node())),
        ));
        let cloned = Rc::clone(&result);

        let mut task = NodeLookupTask::new(id, self.cloned());
        task.set_name("node-lookup");
        task.set_result_fn(move |_task, _node| {
            if _node.is_some() {
                *(result.borrow_mut()) = Some(_node.unwrap().clone());
            }
            if option == LookupOption::Conservative {
                _task.cancel()
            }
        });

        task.add_listener(Box::new(move |_| {
            complete_fn(cloned.borrow_mut().take());
        }));

        self.taskman.borrow_mut().add(
            Rc::new(RefCell::new(task))
        );
    }

    pub(crate) fn find_value<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Box<Value>>) + 'static,
    {
        let result = Rc::new(RefCell::new(Option::default() as Option<Box<Value>>));
        let result_shadow = Rc::clone(&result);

        let mut task = ValueLookupTask::new(self.cloned(), id);
        task.set_name("value-lookup");
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

        task.add_listener(Box::new(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        }));
        self.taskman.borrow_mut().add(
            Rc::new(RefCell::new(task))
        );
    }

    pub(crate) fn store_value<F>(&self, value: &Value, complete_fn: F)
    where F: Fn(Option<Vec<Box<NodeInfo>>>) + 'static
    {
        let mut task = NodeLookupTask::new(&value.id(), self.cloned());
        task.set_name("store-value");
        task.set_want_token(true);
        task.add_listener(Box::new(move |_task| {
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
        }));
        self.taskman.borrow_mut().add(
            Rc::new(RefCell::new(task))
        );
    }

    pub(crate) fn find_peer<F>(&self, id: &Id, expected: usize, option: LookupOption, complete_fn: F)
    where F: Fn(Vec<Box<Peer>>) + 'static
    {
        let result = Rc::new(RefCell::new(Vec::new()));
        let cloned = Rc::clone(&result);

        let mut task = PeerLookupTask::new(id, self.cloned());
        task.set_name("peer-lookup");
        task.set_result_fn(move |_task, _peers| {
            result.borrow_mut().append(_peers);
            if option != LookupOption::Conservative && result.borrow().len() >= expected {
                _task.cancel()
            }
        });

        task.add_listener(Box::new(move |_| {
            complete_fn(cloned.borrow_mut().drain(..).collect());
        }));

        self.taskman.borrow_mut().add(
            Rc::new(RefCell::new(task))
        );
    }

    pub(crate) fn announce_peer<F>(&self, _: &Peer, _: F)
    where
        F: Fn(&[&NodeInfo]),
    {
        unimplemented!()
    }
}
