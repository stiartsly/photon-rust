use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::any::Any;
use std::collections::HashMap;
use std::time::SystemTime;
use log::{error, info, debug};

use crate::{
    constants,
    id::Id,
    node_info::{NodeInfo, Reachable},
    rpccall::RpcCall,
    routing_table::RoutingTable,
    kclosest_nodes::KClosestNodes,
    error::Error,
};

use crate::msg::{
    find_node_req,
    find_node_rsp,
    msg::{self, Msg},

};

use super::{
    candidate_node::CandidateNode,
    closest_candidates::ClosestCandidates,
    closest_set::ClosestSet,
    task::{State, Task, Lookup},
};

#[allow(dead_code)]
pub(crate) struct NodeLookupTask {
    routing_table: Rc<RefCell<RoutingTable>>,
    taskid: i32,
    name: String,
    state: State,

    // nested: *const libc::c_void,
    started_time: SystemTime,
    finished_time: SystemTime,

    inflights: HashMap<usize, Box<RpcCall>>,
    listeners: Vec<Box<dyn FnOnce(&dyn Task)>>,

    // Lookup
    target: Id,
    closest_set: ClosestSet,
    closest_candidates: ClosestCandidates,

    // NodeLookup
    bootstrap: bool,
    want_token: bool,
    result_fn: Box<dyn FnMut(&mut dyn Task, Option<Box<NodeInfo>>)>,
}

#[allow(dead_code)]
impl NodeLookupTask {
    pub(crate) fn new(target: &Id, routing_table: Rc<RefCell<RoutingTable>>) -> Self {
        NodeLookupTask {
            routing_table,
            taskid: 0,
            name: String::from("N/A"),
            state: State::Initial,
            started_time: SystemTime::UNIX_EPOCH,
            finished_time: SystemTime::UNIX_EPOCH,
            inflights: HashMap::new(),
            listeners: Vec::new(),

            target: target.clone(),
            closest_set: ClosestSet::new(target, constants::MAX_ENTRIES_PER_BUCKET),
            closest_candidates: ClosestCandidates::new(
                target,
                3 * constants::MAX_ENTRIES_PER_BUCKET,
            ),

            bootstrap: false,
            want_token: false,
            result_fn: Box::new(|_, _| {}),
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(&mut dyn Task, Option<Box<NodeInfo>>) + 'static,
    {
        self.result_fn = Box::new(f);
    }

    fn remove_listener<F>(&mut self) {
        self.listeners.pop();
    }

    fn remove_candidate(&mut self, id: &Id) -> Option<Box<CandidateNode>> {
        self.closest_candidates.remove(id)
    }

    fn add_closest(&mut self, candidate_node: Box<CandidateNode>) {
        self.closest_set.add(candidate_node);
    }

    pub(crate) fn set_bootstrap(&mut self, bootstrap: bool) {
        self.bootstrap = bootstrap
    }

    pub(crate) fn set_want_token(&mut self, token: bool) {
        self.want_token = token;
    }

    fn can_request(&self) -> bool {
        self.inflights.len() < 10 && !self.is_finished()
    }

    fn prepare(&mut self) {
        // if we're bootstrapping start from the bucket that has the greatest
        // possible distance from ourselves so we discover new things along
        // the (longer) path.
        let target = match self.bootstrap {
            true => self.target.distance(&Id::max()),
            false => self.target.clone()
        };

        // delay the filling of the todo list until we actually start the task
        let mut kclosest_nodes = KClosestNodes::new_with_filter(
            &target,
            Rc::clone(&self.routing_table),
            constants::MAX_ENTRIES_PER_BUCKET *2,
            move |e| e.is_eligible_for_nodes_list()
        );
        kclosest_nodes.fill(false);
        let nodes = kclosest_nodes.as_nodes();
        self.add_candidates(&nodes);
    }

    fn update(&mut self) {
        while self.can_request() {
            if let None = self.next_candidate() {
                break;
            }

            let mut req = Box::new(find_node_req::Message::new());
            req.with_target(self.target().clone());
            req.with_want4(true);
            req.with_want6(false);

            let cn = self.next_candidate().unwrap().clone();
            if let Err(err) = self.send_call(&cn, req, Box::new(|_| {
                //cn.set_sent();
            })) {
                error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }

    fn finish(&mut self) {
        let expected = vec![
            State::Initial,
            State::Queued,
            State::Running
        ];
        if self.set_state(&expected, State::Finished) {
            self.finished_time = SystemTime::now();
            info!("Task finished: {}", self);
            self.notify_completion();
        }
    }

    fn call_sent(&mut self, _: &Box<RpcCall>) {}

    fn call_responsed(&mut self, call: &Box<RpcCall>, rsp: &Box<dyn Msg>) {
        if let Some(mut cn) = self.remove_candidate(call.target_id()) {
            cn.set_replied();
            cn.set_token(1);
            self.add_closest(cn);
        }

        if !call.matches_id()||
            rsp.kind() != msg::Kind::Response ||
            rsp.method() != msg::Method::FindNode {
            return;
        }

        if let Some(downcasted) = rsp.as_any().downcast_ref::<find_node_rsp::Message>() {
            let nodes = downcasted.nodes4(); // TODO:
            if !nodes.is_empty() {
                self.add_candidates(nodes);
            }

            for item in nodes.iter() {
                if item.id() == &self.target {
                    //TODO: (self.result_fn)(self.clone(), None)
                }
            }
        }
    }

    fn call_error(&mut self, call: &Box<RpcCall>) {
        _ = self.closest_candidates.remove(call.target_id())
    }

    fn call_timeout(&mut self, call: &Box<RpcCall>) {
        let mut candidate_node = Box::new(CandidateNode::new(call.target(), false));
        if candidate_node.unreachable() {
            self.closest_candidates.remove(candidate_node.nodeid());
            return;
        }
        // Clear the sent time-stamp and make it available again for the next retry
        candidate_node.clear_sent()
    }

    fn is_done(&self) -> bool {
        self.inflights.is_empty() || self.is_finished()
    }

    fn serialized_update(&mut self) {
        if self.is_done() {
            self.finish();
        }

        if self.can_request() {
            self.update();
            if self.is_done() {
                self.finish()
            }
        }
    }

    fn notify_completion(&mut self) {
        while let Some(f) = self.listeners.pop() {
            f(self as &dyn Task)
        }
        println!("notify_completion");
    }

    fn send_call(&mut self, _: &Box<CandidateNode>, _: Box<dyn Msg>, _: Box<dyn FnOnce(&Box<RpcCall>)>)
        -> Result<(), Error> {
        println!("send call>>>>>>");
        Ok(())
    }
}

impl Task for NodeLookupTask {
    fn taskid(&self) -> i32 {
        self.taskid
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string()
    }

    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, expected: &[State], state: State) -> bool {
        match expected.contains(&self.state) {
            true => { self.state = state; true},
            false => false,
        }
    }

    fn nested(&self) -> &Box<dyn Task> {
        unimplemented!()
    }

    fn set_nested(&mut self, _: Box<dyn Task>) {
        unimplemented!()
    }

    fn is_canceled(&self) -> bool {
        self.state == State::Canceled
    }

    fn is_finished(&self) -> bool {
        self.state == State::Finished
    }

    fn add_listener(&mut self, f: Box<dyn FnOnce(&dyn Task)>) {
        self.listeners.push(f)
    }

    fn start(&mut self) {
        let expected = vec![
            State::Initial,
            State::Queued
        ];
        if self.set_state(&expected, State::Running) {
            self.started_time = SystemTime::now();
            self.prepare();
            self.serialized_update();
        }
    }

    fn cancel(&mut self) {
        let expected = vec![
            State::Initial,
            State::Queued,
            State::Running
        ];
        if self.set_state(expected.as_slice(), State::Canceled) {
            self.finished_time = SystemTime::now();
            debug!("Task canceled: {}", self);
            self.notify_completion();
        }

        // if (!!self.nested)
        //    nested.cancel()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Lookup for NodeLookupTask {
    fn target(&self) -> &Id {
        &self.target
    }

    fn candidate_node(&self, id: &Id) -> Option<&Box<CandidateNode>>  {
        self.closest_candidates.get(id)
    }

    fn closest_set(&self) -> &ClosestSet {
        &self.closest_set
    }

    fn add_candidates(&mut self, nodes: &[NodeInfo]) {
        let mut candidates = Vec::new();

        for item in nodes.iter() {
            if is_bogon_address(item.socket_addr()) ||
                self.routing_table.borrow().node_id() == item.id() ||
                self.routing_table.borrow().node_addr() == item.socket_addr() ||
                self.closest_set.contains(item.id()) {
                continue;
            }
            candidates.push(item.clone());
        }

        if !candidates.is_empty() {
            self.closest_candidates.add(candidates.as_slice())
        }
    }

    fn remove_candidate(&mut self, id: &Id) {
        _ = self.closest_candidates.remove(id)
    }

    fn next_candidate(&self) -> Option<&Box<CandidateNode>> {
        self.closest_candidates.next()
    }

    fn add_closest(&mut self, candidate_node: Box<CandidateNode>) {
        self.closest_set.add(candidate_node)
    }
}

impl fmt::Display for NodeLookupTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{}[{}] DHT:{}, state:{}",
            self.taskid,
            self.name.as_str(),
            "ipv4<TODO>",
            self.state
        )?;
        Ok(())
    }
}

fn is_bogon_address(_: &SocketAddr) -> bool {
    false
}
