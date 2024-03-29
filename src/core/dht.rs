
use std::rc::Rc;
use std::collections::{LinkedList, HashMap};
use std::cell::RefCell;
use std::time::SystemTime;
use std::net::SocketAddr;
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
};

use crate::msg::{
    ping_req,
    find_node_req,
    find_node_rsp,
    find_peer_rsp,
    store_value_rsp,
    find_value_rsp,
    announce_peer_rsp,
    error,
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
    persist_path: Option<String>,
    last_save: SystemTime,
    running: bool,

    bootstrap_need: bool,
    bootstrap_nodes: LinkedList<Box<NodeInfo>>,
    bootstrap_time: SystemTime,
    //bootstrap_flag: AtomicBool,

    next_tid: i32,
    calls: HashMap<i32, Box<RpcCall>>,

    routing_table: RoutingTable,

    server:     Rc<RefCell<Server>>,
    taskman:   Rc<RefCell<TaskManager>>,
    token_man:  Rc<RefCell<TokenManager>>,
    scheduler:  Rc<RefCell<Scheduler>>,

    queue: Rc<RefCell<LinkedList<Box<dyn Msg>>>>,
}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(server: &Rc<RefCell<Server>>, binding_addr: SocketAddr) -> Self {
        DHT {
            addr: binding_addr,
            running: false,
            persist_path: None,
            last_save: SystemTime::UNIX_EPOCH,
            routing_table: RoutingTable::new(),

            bootstrap_nodes: LinkedList::new(),
            bootstrap_need: false,
            bootstrap_time: SystemTime::UNIX_EPOCH,
            // bootstrap_flag: AtomicBool::new(false),

            next_tid: 0,
            calls: HashMap::new(),

            server: Rc::clone(server),
            taskman:  Rc::new(RefCell::new(TaskManager::new())),
            token_man: Rc::clone(server.borrow().tokenman()),
            scheduler: Rc::clone(server.borrow().scheduler()),

            queue: Rc::new(RefCell::new(LinkedList::new())),
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_path = Some(String::from(path));
    }

    pub(crate) fn add_bootstrap(&mut self, node: Box<NodeInfo>) {
        self.bootstrap_nodes.push_back(node);
        self.bootstrap_need = true;
    }

    pub(crate) fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub(crate) fn queue(&self) -> Rc<RefCell<LinkedList<Box<dyn Msg>>>> {
        Rc::clone(&self.queue)
    }

    pub(crate) fn is_ipv4(&self) -> bool {
        self.addr.is_ipv4()
    }

    pub(crate) fn bootstrap(&mut self) {
        let mut nodes = self.bootstrap_nodes.clone().into_iter().collect::<Vec<_>>();
        if nodes.is_empty() {
            nodes = self.routing_table.random_entries(8);
        }

        info!("DHT/{} bootstraping ....", as_kind_name!(&self.addr));

       /* if self.bootstrap_flag.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            return;
        }*/

        nodes.iter().for_each(|item| {
            let mut req = Box::new(find_node_req::Message::new());
            req.set_id(item.id());
            req.set_addr(item.socket_addr());
            req.with_target(&Id::random());
            match self.is_ipv4() {
                true  => req.with_want4(),
                false => req.with_want6(),
            };

            let mut call = Box::new(RpcCall::new(item, req));
            call.set_state_changed_fn(move |call, _, cur| {
                if cur == &rpccall::State::Responsed || cur == &rpccall::State::Err ||
                    cur == &rpccall::State::Timeout {

                    if let Some(_) = call.rsp() {
                        //let list = resp.nodes();
                        println!("resp: {}", "response");
                    }
                    // TODO:
                }
            });
            self.send_call(call);
        });
    }

    fn fill_home_bucket(&mut self, _: &[NodeInfo]) {
        unimplemented!()
    }

    pub(crate) fn update(&mut self) {
        if !self.is_running() {
            return;
        }

        trace!("DHT/{} regularly update...", as_kind_name!(&self.addr));
        //self.server.borrow_mut().update_reachability();
        self.routing_table.maintenance();

        if self.bootstrap_need || self.routing_table.size() < constants::BOOTSTRAP_IF_LESS_THAN_X_PEERS ||
            as_millis!(self.bootstrap_time) > constants::SELF_LOOKUP_INTERVAL {

            self.bootstrap_need = false;
            // Regularly search for our ID to update the routing table
            self.bootstrap();
        }

        if as_millis!(self.last_save) > constants::ROUTING_TABLE_PERSIST_INTERVAL {
            info!("Persisting routing table ....");
            self.routing_table.save(self.persist_path.as_ref().unwrap().as_str());
            self.last_save = SystemTime::now();
        }
    }

    pub(crate) fn start(&mut self) {
        if self.is_running() {
            return;
        }

        // Load neighboring nodes from cache storage if they exist.
        if let Some(path) = self.persist_path.as_ref() {
            info!("Loading routing table from [{}] ...", path);
            self.routing_table.load(path);
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

        // Regularly DHT update
        self.scheduler.borrow_mut().add(move || {
            // TODO;
        }, 100, constants::DHT_UPDATE_INTERVAL);

        // Send a ping request to a random node to verify socket liveness.
        self.scheduler.borrow_mut().add(move || {
            // TODO;
        }, constants::RANDOM_PING_INTERVAL, constants::RANDOM_PING_INTERVAL);

        // Perform a deep lookup to familiarize ourselves with random sections of
        // the keyspace.
        //let mut kind = String::from(addr_kind(&self.addr));
        let addr = self.addr.clone();
        let taskman = Rc::clone(&self.taskman);
        self.scheduler.borrow_mut().add(move || {
            let mut task = Box::new(NodeLookupTask::new(&Id::random()));
            let name = format!("{}: random lookup", as_kind_name!(&addr));
            task.with_name(&name);
            task.add_listener(move |_|{});
            taskman.borrow_mut().add(task);
        }, constants::RANDOM_LOOKUP_INTERVAL, constants::RANDOM_LOOKUP_INTERVAL)
    }

    pub(crate) fn stop(&mut self) {
        if !self.is_running() {
            return;
        }

        info!("{} initiated shutdown ...", as_kind_name!(&self.addr));
        info!("stopping servers ...");

        self.running = false;

        info!("Persisting routing table on shutdown ...");
        if let Some(path) = self.persist_path.as_ref() {
            self.routing_table.save(path);
        }
        self.taskman.borrow_mut().cancel_all();
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running
    }

    pub(crate) fn on_msg(&mut self, msg: Rc<dyn Msg>)
    {
        match msg.kind() {
            msg::Kind::Error => self.on_error(msg),
            msg::Kind::Request => self.on_request(msg),
            msg::Kind::Response => self.on_response(msg),
        }
    }

    fn on_request(&mut self, msg: Rc<dyn Msg>)
    {
        match msg.method() {
            msg::Method::Ping => self.on_ping(msg),
            msg::Method::FindNode => self.on_find_node(msg),
            msg::Method::FindValue => self.on_find_value(msg),
            msg::Method::StoreValue => self.on_store_value(msg),
            msg::Method::FindPeer => self.on_find_peers(msg),
            msg::Method::AnnouncePeer => self.on_announce_peer(msg),
            msg::Method::Unknown => {
                self.send_err(msg, 203, "Invalid request method");
            }
        }
    }

    fn on_response(&mut self, _: Rc<dyn Msg>) {}

    fn on_error(&mut self, msg: Rc<dyn Msg>)
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

    fn send_err(&mut self, msg: Rc<dyn Msg>, code: i32, str: &str) {
        let mut err = Box::new(error::Message::new());

        err.set_id(msg.id());
        err.set_addr(msg.addr());
        err.set_ver(version::build(version::NODE_TAG_NAME, version::NODE_VERSION));
        err.set_txid(msg.txid());
        err.with_msg(str);
        err.with_code(code);

        self.send_msg(err);
    }

    fn on_ping(&mut self, request: Rc<dyn Msg>) {
        let mut msg = Box::new(ping_req::Message::new());

        msg.set_id(request.id());
        msg.set_addr(request.addr());
        msg.set_txid(request.txid());

        self.send_msg(msg);
    }

    fn on_find_node<'a>(&mut self, request: Rc<dyn Msg>) {
        let mut resp = Box::new(find_node_rsp::Message::new());

        resp.set_id(request.id());
        resp.set_txid(request.txid());
        resp.set_addr(request.addr());

        if request.want4() {
            let cloned = Rc::clone(&request);
            resp.populate_closest_nodes4(Box::new(move || {
                let mut knodes = KClosestNodes::new(
                    cloned.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                );
                knodes.fill(true);
                knodes.as_nodes()
            }));
        }
        if request.want6() {
            let cloned = Rc::clone(&request);
            resp.populate_closest_nodes6(Box::new(move || {
                let mut knodes = KClosestNodes::new(
                    cloned.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                );
                knodes.fill(true);
                knodes.as_nodes()
            }));
        }

        let cloned = Rc::clone(&self.token_man);
        resp.populate_token(request.want_token(), Box::new(move || {
            cloned.borrow()
                .generate_token(request.id(), request.addr(), request.target())
        }));

        self.send_msg(resp)
    }

    fn on_find_value(&mut self, request: Rc<dyn Msg>) {
        let mut resp = Box::new(find_value_rsp::Message::new());

        resp.set_id(request.id());
        resp.set_txid(request.txid());
        resp.set_addr(request.addr());

        let has_value = RefCell::new(false);
        let cloned_has_value = RefCell::clone(&has_value);
        let cloned_request = Rc::clone(&request);
        resp.populate_value(Box::new(move || {
            let value: Option<Box<Value>> = None;
            if value.is_some() {
                if cloned_request.seq() < 0
                    || value.as_ref().unwrap().sequence_number() < 0
                    || cloned_request.seq() <= value.as_ref().unwrap().sequence_number()
                {
                    *cloned_has_value.borrow_mut() = true;
                }
            }
            value
        }));

        if request.want4() && *has_value.borrow() {
            let cloned = Rc::clone(&request);
            resp.populate_closest_nodes4(Box::new(move || {
                KClosestNodes::new(
                    cloned.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes()
            }));
        }

        if request.want6() && *has_value.borrow() {
            let cloned = Rc::clone(&request);
            resp.populate_closest_nodes4(Box::new(move || {
                KClosestNodes::new(
                    cloned.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes()
            }));
        }

        let cloned = Rc::clone(&self.token_man);
        resp.populate_token(request.want_token(), Box::new(move || {
            cloned.borrow()
                .generate_token(request.id(), request.addr(), request.target())
        }));

        self.send_msg(resp);
    }

    fn on_store_value(&mut self, request: Rc<dyn Msg>) {
        let value = request.value();
        let value_id = value.as_ref().unwrap().id();

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
                request,
                203,
                "Invalid token for STORE VALUE request",
            );
            return;
        }

        if !value.as_ref().unwrap().is_valid() {
            self.send_err(request, 203, "Invalid value");
            return;
        }
        // TODO: store value.
        let mut resp = Box::new(store_value_rsp::Message::new());

        resp.set_id(request.id());
        resp.set_addr(request.addr());
        resp.set_txid(request.txid());

        self.send_msg(resp);
    }

    fn on_find_peers(&mut self, request: Rc<dyn Msg>) {
        let mut resp = Box::new(find_peer_rsp::Message::new());

        resp.set_id(request.id());
        resp.set_addr(request.addr());
        resp.set_txid(request.txid());

        let has_peers = RefCell::new(false);
        let cloned_has_peers = RefCell::clone(&has_peers);
        resp.populate_peers(Box::new(move || {
            // TODO;
            let peers: Vec<Box<Peer>> = Vec::new();
            if !peers.is_empty() {
                *cloned_has_peers.borrow_mut() = true;
            };
            Some(peers)
        }));

        if request.want4() && *has_peers.borrow() {
            let cloned = Rc::clone(&request);
            resp.populate_closest_nodes4(Box::new(move || {
                KClosestNodes::new(
                    cloned.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes()
            }));
        }

        if request.want6() && *has_peers.borrow() {
            let cloned = Rc::clone(&request);
            resp.populate_closest_nodes4(Box::new(move || {
                KClosestNodes::new(
                    cloned.target(),
                    constants::MAX_ENTRIES_PER_BUCKET,
                )
                .fill(true)
                .as_nodes()
            }));
        }

        let cloned = Rc::clone(&self.token_man);
        resp.populate_token(request.want_token(), Box::new(move || {
            cloned.borrow()
                .generate_token(request.id(), request.addr(), request.target())
        }));

        self.send_msg(resp);
    }

    fn on_announce_peer(&mut self, request: Rc<dyn Msg>) {
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
                request,
                203,
                "Invalid token for ANNOUNCE PEER request",
            );
            return;
        }

        let peers = request.peers();
        for peer in peers.iter() {
            if !peer.is_valid() {
                self.send_err(request, 203, "One peer is invalid peer");
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

        resp.set_id(request.id());
        resp.set_addr(request.addr());
        resp.set_txid(request.txid());

        self.send_msg(resp);
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

        self.taskman.borrow_mut().add(task);
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
        self.taskman.borrow_mut().add(task);
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

        let mut task = Box::new(PeerLookupTask::new(id));
        task.with_name("peer-lookup");
        task.set_result_fn(move |_task, _peers| {
            (*result.borrow_mut()).append(_peers);
            if option != LookupOption::Conservative && result.borrow().len() >= expected {
                _task.cancel()
            }
        });

        task.add_listener(move |_| complete_fn(result_shadow.take()));

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
        if let Some(mut call) = msg.associated_call() {
            call.dht().borrow_mut().on_send(call.target_id());
            call.send(&self.scheduler);
        }

        self.queue.borrow_mut().push_back(msg);
    }

    pub(crate) fn send_call(&mut self, mut call: Box<RpcCall>) {
        self.next_tid += 1;
        let mut txid = self.next_tid;
        if txid == 0 {
            txid += 1;
        }

        call.set_responsed_fn(|_,_| {});
        call.set_timeout_fn(|_|{});

        let mut req = call.req();
        req.set_txid(txid);
        // call.req_mut().with_associated_call(&call);
        self.calls.insert(txid, call);

        self.send_msg(req);
    }
}
