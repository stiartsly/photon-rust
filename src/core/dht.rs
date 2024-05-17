
use std::rc::Rc;
use std::collections::{LinkedList, HashMap};
use std::cell::RefCell;
use std::time::SystemTime;
use std::net::SocketAddr;
use std::ops::Deref;
use log::{debug, info, warn, trace};

use crate::{
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
    scheduler::Scheduler,
    kbucket_entry::KBucketEntry,
};

use crate::msg::{
    ping_req,
    ping_rsp,
    find_node_req,
    find_node_rsp,
    find_peer_rsp,
    store_value_rsp,
    find_value_rsp,
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

    calls: HashMap<i32, Rc<RefCell<RpcCall>>>,

    routing_table: Rc<RefCell<RoutingTable>>,
    server: Rc<RefCell<Server>>,
    taskman: Rc<RefCell<TaskManager>>,
    tokenman: Rc<RefCell<TokenManager>>,
    scheduler: Rc<RefCell<Scheduler>>,

    queue: Rc<RefCell<LinkedList<Box<dyn Msg>>>>,
}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(server: Rc<RefCell<Server>>, binding_addr: SocketAddr) -> Self {
        DHT {
            nodeid: server.borrow().nodeid().clone(),
            addr: binding_addr,
            running: false,
            persist_path: None,
            last_save: SystemTime::UNIX_EPOCH,
            routing_table: Rc::new(RefCell::new(RoutingTable::new(server.borrow().nodeid(), &binding_addr))),

            bootstrap_nodes: Vec::new(),
            bootstrap_need: false,
            bootstrap_time: Rc::new(RefCell::new(SystemTime::UNIX_EPOCH)),

            calls: HashMap::new(),

            server: Rc::clone(&server),
            taskman:  Rc::new(RefCell::new(TaskManager::new())),
            tokenman: Rc::clone(server.borrow().tokenman()),
            scheduler: Rc::clone(server.borrow().scheduler()),

            queue: Rc::new(RefCell::new(LinkedList::new())),
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_path = Some(String::from(path));
    }

    pub(crate) fn add_bootstrap_node(&mut self, node: Box<NodeInfo>) {
        self.bootstrap_nodes.push(node);
        self.bootstrap_need = true;
    }

    pub(crate) fn socket_addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub(crate) fn nodeid(&self) -> &Id {
        &self.nodeid
    }

    pub(crate) fn routing_table(&self) -> Rc<RefCell<RoutingTable>> {
        Rc::clone(&self.routing_table)
    }

    pub(crate) fn queue(&self) -> Rc<RefCell<LinkedList<Box<dyn Msg>>>> {
        Rc::clone(&self.queue)
    }

    pub(crate) fn bootstrap(&mut self) {
        let bns = match self.bootstrap_nodes.is_empty() {
            true => self.routing_table.borrow().random_entries(8),
            false => self.bootstrap_nodes.clone()
        };

        info!("DHT/{} bootstraping ....", as_kind_name!(&self.addr));

        let nodes = Rc::new(RefCell::new(Vec::new())) as Rc<RefCell<Vec<NodeInfo>>>;
        let count = Rc::new(RefCell::new(0));

        for node in bns.iter() {
            let mut req = Box::new(find_node_req::Message::new());
            req.set_id(node.id().clone());
            req.set_addr(node.socket_addr().clone());
            req.with_target(Id::random());
            req.with_want4(true);

            let call = Rc::new(RefCell::new(RpcCall::new(node.clone(), req)));
            let len = bns.len();
            let cloned_nodes = Rc::clone(&nodes);
            let cloned_count = Rc::clone(&count);
            let cloned_id = Rc::new(node.id().clone());
            let taskman = Rc::clone(&self.taskman);
            let routing_table = Rc::clone(&self.routing_table);
            let bootstrap_time = Rc::clone(&self.bootstrap_time);
            let server = Rc::clone(&self.server);
            call.borrow_mut().set_state_changed_fn(move |call, _, cur| {
                if cur == &rpccall::State::Responsed || cur == &rpccall::State::Err ||
                    cur == &rpccall::State::Timeout {
                    if let Some(rsp) = call.rsp() {
                        cloned_nodes.borrow_mut().extend_from_slice(rsp.nodes4());
                    }


                    *cloned_count.borrow_mut() += 1;
                    if *cloned_count.borrow() == len {

                        *bootstrap_time.borrow_mut() = SystemTime::now();
                        if routing_table.borrow().size() == 0 &&
                            cloned_nodes.borrow().is_empty() {
                            return;
                        }


                        let task = Rc::new(RefCell::new(NodeLookupTask::new(
                            cloned_id.deref(),
                            Rc::clone(&routing_table)
                        )));
                        let cloned_task = Rc::clone(&task);
                        task.borrow_mut().link_self(cloned_task);
                        task.borrow_mut().link_server(Rc::clone(&server));
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

            self.send_call(call);
        };
    }

    pub(crate) fn update(&mut self) {
        if !self.running {
            return;
        }

        trace!("DHT/{} regularly update...", as_kind_name!(&self.addr));
        //self.server.borrow_mut().update_reachability();
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
        let taskman = Rc::clone(&self.taskman);
        self.scheduler.borrow_mut().add(move || {
            taskman.borrow_mut().dequeue();
        }, 500, constants::DHT_UPDATE_INTERVAL);

        // fix the first time to persist the routing table: 2 min
        //lastSave = currentTimeMillis() - Constants::ROUTING_TABLE_PERSIST_INTERVAL + (120 * 1000);

        // Send a ping request to a random node to verify socket liveness.
        self.scheduler.borrow_mut().add(move || {
            // TODO;
        }, constants::RANDOM_PING_INTERVAL, constants::RANDOM_PING_INTERVAL);

        // Perform a deep lookup to familiarize ourselves with random sections of
        // the keyspace.
        let addr = self.addr.clone();
        let taskman = Rc::clone(&self.taskman);
        let routing_table = Rc::clone(&self.routing_table);
        let server = Rc::clone(&self.server);
        self.scheduler.borrow_mut().add(move || {
            let task = Rc::new(RefCell::new(NodeLookupTask::new(&Id::random(), Rc::clone(&routing_table))));
            let name = format!("{}: random lookup", as_kind_name!(&addr));
            let task_cloned = Rc::clone(&task);
            task.borrow_mut().link_self(task_cloned);
            task.borrow_mut().link_server(Rc::clone(&server));
            task.borrow_mut().set_name(&name);
            task.borrow_mut().add_listener(Box::new(move |_|{}));

            taskman.borrow_mut().add(task);
        }, constants::RANDOM_LOOKUP_INTERVAL, constants::RANDOM_LOOKUP_INTERVAL)
    }

    pub(crate) fn stop(&mut self) {
        if !self.running {
            return;
        }

        info!("{} initiated shutdown ...", as_kind_name!(&self.addr));
        info!("stopping servers ...");

        self.running = false;

        info!("Persisting routing table on shutdown ...");
        if let Some(path) = self.persist_path.as_ref() {
            self.routing_table.borrow_mut().save(path);
        }
        self.taskman.borrow_mut().cancel_all();
    }

    pub(crate) fn on_message(&mut self, msg: Box<dyn Msg>) {
        let msg = self.responsed(msg);
        match msg.kind() {
            Kind::Error => self.on_error(&msg),
            Kind::Request => self.on_request(&msg),
            Kind::Response => self.on_response(&msg),
        };
        self.received(msg);
    }

    fn responsed(&mut self, mut msg: Box<dyn Msg>) -> Box<dyn Msg> {
        match self.calls.remove(&msg.txid()) {
            Some(call) => {
                msg.with_associated_call(Rc::clone(&call));
                call.borrow_mut().responsed(msg)
            },
            None => msg
        }
    }

    fn received(&mut self, msg: Box<dyn Msg>) {
        let from_id = msg.id();
        let from_addr = msg.addr();

        if is_bogon_address(from_addr) {
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
            new_entry.set_version(msg.version());
            new_entry.signal_response();
            new_entry.merge_request_time(call.borrow().sent_time().clone());
        } else if !entry_found {
            let mut req = Box::new(ping_req::Message::new());
            req.set_id(msg.id().clone());
            req.set_addr(from_addr.clone());

            let ni = Box::new(new_entry.inner_node());
            let call = Rc::new(RefCell::new(RpcCall::new(ni, req)));
            self.send_call(call);
        }
        self.routing_table.borrow_mut().put(new_entry);
    }

    fn on_request(&mut self, msg: &Box<dyn Msg>) {
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

    fn on_response(&mut self, _: &Box<dyn Msg>) {}

    fn on_error(&mut self, msg: &Box<dyn Msg>) {
        warn!("Error from {}/{} - {}:{}, txid {}",
            msg.addr(),
            version::formatted_version(msg.version()),
            msg.code(),
            msg.msg(),
            msg.txid()
        );
    }

    fn send_err(&mut self, msg: &Box<dyn Msg>, code: i32, str: &str) {
        let mut err = Box::new(error::Message::new());

        err.set_id(msg.id().clone());
        err.set_addr(msg.addr().clone());
        err.set_ver(version::build(version::NODE_TAG_NAME, version::NODE_VERSION));
        err.set_txid(msg.txid());
        err.with_msg(str);
        err.with_code(code);

        self.send_msg(err);
    }

    fn on_ping(&mut self, req: &Box<dyn Msg>) {
        let mut rsp = Box::new(ping_rsp::Message::new());
        rsp.set_id(req.id().clone());
        rsp.set_addr(req.addr().clone());
        rsp.set_txid(req.txid());

        self.send_msg(rsp);
    }

    fn on_find_node(&mut self, req: &Box<dyn Msg>) {
        let mut rsp = Box::new(find_node_rsp::Message::new());
        rsp.set_id(req.id().clone());
        rsp.set_addr(req.addr().clone());
        rsp.set_txid(req.txid());

        if req.want4() {
            let mut knodes = KClosestNodes::new(
                req.target(),
                Rc::clone(&self.routing_table),
                constants::MAX_ENTRIES_PER_BUCKET,
            );
            knodes.fill(true);
            rsp.populate_closest_nodes4(knodes.as_nodes());
        }

        if req.want_token() {
            let token = self.tokenman.borrow_mut().generate_token(
                req.id(), req.addr(), req.target()
            );
            rsp.populate_token(token);
        }

        self.send_msg(rsp)
    }

    fn on_find_value(&mut self, req: &Box<dyn Msg>) {
        let mut rsp = Box::new(find_value_rsp::Message::new());
        rsp.set_id(req.id().clone());
        rsp.set_addr(req.addr().clone());
        rsp.set_txid(req.txid());

        let mut has_value = false;
        let value = self.server.borrow().storage().borrow().value(req.target());
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
                Rc::clone(&self.routing_table),
                constants::MAX_ENTRIES_PER_BUCKET,
            );
            knodes.fill(true);
            rsp.populate_closest_nodes4(knodes.as_nodes());
        }

        if req.want_token() {
            let token = self.tokenman.borrow_mut().generate_token(
                req.id(), req.addr(), req.target()
            );
            rsp.populate_token(token);
        }

        self.send_msg(rsp);
    }

    fn on_store_value(&mut self, req: &Box<dyn Msg>) {
        let value = req.value();
        let value_id = value.as_ref().unwrap().id();

        let valid = self.tokenman.borrow_mut().verify_token(
            req.token(), req.id(), req.addr(), &value_id,
        );
        if !valid {
            warn!(
                "Received a store value request with invalid token from {}",
                req.addr()
            );
            self.send_err(
                req,
                203,
                "Invalid token for STORE VALUE request",
            );
            return;
        }

        if !value.as_ref().unwrap().is_valid() {
            self.send_err(req, 203, "Invalid value");
            return;
        }
        // TODO: store value.
        let mut rsp = Box::new(store_value_rsp::Message::new());
        rsp.set_id(req.id().clone());
        rsp.set_addr(req.addr().clone());
        rsp.set_txid(req.txid());

        self.send_msg(rsp);
    }

    fn on_find_peers(&mut self, req: &Box<dyn Msg>) {
        let mut rsp = Box::new(find_peer_rsp::Message::new());
        rsp.set_id(req.id().clone());
        rsp.set_addr(req.addr().clone());
        rsp.set_txid(req.txid());

        let mut has_peers = false;
        let peers = self.server.borrow().storage().borrow().peers(req.target(), 8);
        if !peers.is_empty() {
            has_peers = true;
            rsp.populate_peers(peers);
        }

        if req.want4() && has_peers {
            let mut knodes = KClosestNodes::new(
                req.target(),
                Rc::clone(&self.routing_table),
                constants::MAX_ENTRIES_PER_BUCKET,
            );
            knodes.fill(true);
            rsp.populate_closest_nodes4(knodes.as_nodes());
        }

        if req.want_token() {
            let token = self.tokenman.borrow_mut().generate_token(
                req.id(), req.addr(), req.target()
            );
            rsp.populate_token(token);
        }

        self.send_msg(rsp);
    }

    fn on_announce_peer(&mut self, req: &Box<dyn Msg>) {
        if is_bogon_address(req.addr()) {
            info!(
                "Received an announce peer request from bogon address {}, ignored ",
                req.addr()
            );
        }

        let valid = self.tokenman.borrow_mut().verify_token(
            req.token(), req.id(), req.addr(), req.target()
        );
        if !valid {
            warn!(
                "Received an announce peer request with invalid token from {}",
                req.addr()
            );
            self.send_err(
                req, 203,"Invalid token for ANNOUNCE PEER request",
            );
            return;
        }

        let peers = req.peers();
        for peer in peers.iter() {
            if !peer.is_valid() {
                self.send_err(req, 203, "One peer is invalid peer");
                return;
            }
        }

        debug!(
            "Received an announce peer request from {}, saving peer {}",
            req.addr(),
            req.target()
        );
        // TODO: Store peers.

        let mut rsp = Box::new(announce_peer_rsp::Message::new());
        rsp.set_id(req.id().clone());
        rsp.set_addr(req.addr().clone());
        rsp.set_txid(req.txid());

        self.send_msg(rsp);
    }

    pub(crate) fn on_timeout(&mut self, call: &RpcCall) {
        // ignore the timeout if the DHT is stopped or the RPC server is offline
        if !self.running || !self.server.borrow().is_reachable() {
            return;
        }
        self.routing_table.borrow_mut().on_timeout(call.target_id());
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
        let result_shadow = Rc::clone(&result);

        let task = Rc::new(RefCell::new(NodeLookupTask::new(id, Rc::clone(&self.routing_table))));
        task.borrow_mut().set_name("node-lookup");
        task.borrow_mut().set_result_fn(move |_task, _node| {
            if _node.is_some() {
                *(result.borrow_mut()) = Some(_node.unwrap().clone());
            }
            if option == LookupOption::Conservative {
                _task.cancel()
            }
        });
        task.borrow_mut().add_listener(Box::new(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        }));

        self.taskman.borrow_mut().add(task);
    }

    pub(crate) fn find_value<F>(&self, id: &Id, option: LookupOption, complete_fn: F)
    where F: Fn(Option<Box<Value>>) + 'static,
    {
        let result = Rc::new(RefCell::new(Option::default() as Option<Box<Value>>));
        let result_shadow = Rc::clone(&result);

        let task = Rc::new(RefCell::new(ValueLookupTask::new(id)));
        task.borrow_mut().set_name("value-lookup");
        task.borrow_mut().set_result_fn(move |_task, _value| {
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

        task.borrow_mut().add_listener(move |_| {
            complete_fn(result_shadow.borrow_mut().take());
        });
        self.taskman.borrow_mut().add(task);
    }

    pub(crate) fn store_value<F>(&self, value: &Value, complete_fn: F)
    where F: Fn(Option<Vec<Box<NodeInfo>>>) + 'static,
    {
        let task = Rc::new(RefCell::new(NodeLookupTask::new(&value.id(), Rc::clone(&self.routing_table))));
        task.borrow_mut().set_name("store-value");
        task.borrow_mut().set_want_token(true);
        task.borrow_mut().add_listener(Box::new(move |_task| {
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
        self.taskman.borrow_mut().add(task);
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

        let task = Rc::new(RefCell::new(PeerLookupTask::new(id)));
        task.borrow_mut().set_name("peer-lookup");
        task.borrow_mut().set_result_fn(move |_task, _peers| {
            (*result.borrow_mut()).append(_peers);
            if option != LookupOption::Conservative && result.borrow().len() >= expected {
                _task.cancel()
            }
        });

        task.borrow_mut().add_listener(move |_| complete_fn(result_shadow.take()));

        self.taskman.borrow_mut().add(task);
    }

    pub(crate) fn announce_peer<F>(&self, _: &Peer, _: F)
    where
        F: Fn(&[&NodeInfo]),
    {
        unimplemented!()
    }

    fn send_msg(&mut self, msg: Box<dyn Msg>) {
        // Handle associated call if it exists:
        // - Notify Kademlia DHT of being interacting with a neighboring node;
        // - Process some internal state for this RPC call.
        if let Some(call) = msg.associated_call() {
            self.on_send(msg.id());
            call.borrow_mut().send();

            let call = Rc::clone(&call);
            self.scheduler.borrow_mut().add(move || {
               call.borrow_mut().check_timeout()
            }, 2000, 10);
        }

        self.queue.borrow_mut().push_back(msg);
    }

    pub(crate) fn send_call(&mut self, call: Rc<RefCell<RpcCall>>) {
        call.borrow_mut().set_responsed_fn(|_,_| {});
        call.borrow_mut().set_timeout_fn(|_call| {
            // self.on_timeout(_call);
        });

        let cloned_call = Rc::clone(&call);
        let msg = call.borrow_mut().req();
        let txid = call.borrow_mut().hash();
        self.calls.insert(txid, call);

        if let Some(mut msg) = msg {
            msg.set_txid(txid);
            msg.with_associated_call(cloned_call);
            self.send_msg(msg);
        }
    }
}

fn is_bogon_address(_: &SocketAddr) -> bool {
    false
}
