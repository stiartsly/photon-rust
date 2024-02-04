use std::fmt;
use std::collections::HashMap;
use std::time::SystemTime;
use log::debug;

use crate::{
    constants,
    id::Id,
    node::Node,
    rpccall::RpcCall
};

use crate::msg::{
    msg::{self, Msg},
    lookup::{Result as MsgResult},
    find_node_rsp::{self}
};

use super::{
    task::{Task, State},
    lookup::LookupTask,
    candidate_node::CandidateNode,
    closest_set::ClosestSet,
    closest_candidates::ClosestCandidates
};

#[allow(dead_code)]
pub(crate) struct NodeLookupTask {
    //dht: Rc<&'a DHT>,
    taskid: i32,
    name: String,
    state: State,

    // nested: *const libc::c_void,

    started_time: SystemTime,
    finished_time: SystemTime,

    inflights: HashMap<usize, Box<RpcCall>>,
    listeners: Vec<Box<dyn FnMut(&Box<dyn Task>)>>,

    // Lookup
    target: Id,
    closest_set: ClosestSet,
    closest_candidates: ClosestCandidates,

    // NodeLookup
    bootstrap: bool,
    want_token: bool,
    result_fn: Box<dyn FnMut(&mut dyn Task, Option<Box<Node>>)>,
}

#[allow(dead_code)]
impl NodeLookupTask {
    pub(crate) fn new(target :&Id) -> Self {
        NodeLookupTask { //dht,
            taskid: 0,
            name: String::from("N/A"),
            state: State::Initial,
            started_time: SystemTime::UNIX_EPOCH,
            finished_time: SystemTime::UNIX_EPOCH,
            inflights: HashMap::new(),
            listeners: Vec::new(),

            target: target.clone(),
            closest_set: ClosestSet::new(
                target,
                constants::MAX_ENTRIES_PER_BUCKET
            ),
            closest_candidates: ClosestCandidates::new(
                target,
                3 * constants::MAX_ENTRIES_PER_BUCKET
            ),

            bootstrap: false,
            want_token: false,
            result_fn: Box::new(|_,_|{})
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(&mut dyn Task, Option<Box<Node>>) + 'static {
        self.result_fn = Box::new(f);
    }

    pub(crate) fn add_listener<F>(&mut self, f: F)
    where F: FnMut(&Box<dyn Task>) + 'static {
        self.listeners.push(Box::new(f));
    }

    fn remove_listener<F>(&mut self) {
        self.listeners.pop();
    }

    pub(crate) fn with_want_token(&mut self, _: bool) {
        unimplemented!()
    }

    fn remove_candidate(&mut self, id: &Id) ->Option<Box<CandidateNode>> {
        self.closest_candidates.remove(id)
    }

    /*
    Sp<CandidateNode> removeCandidate(const Id& id) {
        return closestCandidates.remove(id);
    }

    Sp<CandidateNode> getNextCandidate() const {
        return closestCandidates.next();
    }

    void addClosest(Sp<CandidateNode> candidateNode) {
        closestSet.add(candidateNode);
    }*/
}

impl Task for NodeLookupTask {
    fn taskid(&self) -> i32 {
        self.taskid
    }

    fn name(&self) -> &str{
        self.name.as_str()
    }
    fn with_name(&mut self, name: &str) {
        self.name = name.to_string()
    }

    fn state(&self) -> State{
        self.state
    }

    fn nested(&self) -> &Box<dyn Task> {
        unimplemented!()
    }

    fn is_canceled(&self) -> bool{
        self.state == State::Canceled
    }

    fn is_finished(&self) -> bool{
        self.state == State::Finished
    }

    fn started_time(&self) -> &SystemTime{
        &self.started_time
    }

    fn finished_time(&self) -> &SystemTime{
        &self.finished_time
    }

    fn age(&self) -> u128 {
        self.started_time.elapsed().unwrap().as_millis()
    }

    fn set_nested(&mut self, _: Box<dyn Task>) {
        unimplemented!()
    }

    fn start(&mut self){
        if match self.state {
            State::Initial => { self.state = State::Running; true },
            State::Queued => { self.state = State::Running; true },
            _ => {false}
        } {
            debug!("Task starting: {}", self);
            self.started_time = SystemTime::now();

            self.prepare();
            //self.serialized_updated();
        }
    }

    fn cancel(&mut self){
        if match self.state {
            State::Initial => { self.state = State::Canceled; true },
            State::Queued => { self.state = State::Canceled; true },
            State::Running => { self.state = State::Canceled; true },
            _ => {false}
        } {
            self.finished_time = SystemTime::now();
            debug!("Task canceled: {}", self);

            // self.notify_completion_listeners();
        }
        // if (!!self.nested)
        //    nested.cancel()
    }

    fn call_sent(&mut self, _: &Box<RpcCall>) {}

    fn call_responsed(&mut self, call: &Box<RpcCall>, rsp: &Box<dyn Msg>) {
        // TODO: LookupTask::callResponsed(xxx)

        if !call.matches_id() ||
            rsp.kind() != msg::Kind::Response ||
            rsp.method() != msg::Method::FindNode {
            return
        }

        match rsp.as_any().downcast_ref::<find_node_rsp::Message>() {
            Some(downcasted) => {
                let nodes = downcasted.nodes4(); // TODO:
                if !nodes.is_empty() {
                    self.add_candidates(&nodes);
                }

                nodes.iter().for_each(|item| {
                    if item.id() == self.target() {
                        //self.result_fn.unwrap()(Some(Box::new(item.clone())), self as &mut dyn Task);
                    }
                })
            }
            None => {
                panic!("panic on powncasting to find_node_response msg")
            }
        }
    }

    fn call_error(&mut self, _: &Box<RpcCall>) {
        //self.as_closest_candidates().remove(call.id())
    }

    fn call_timeout(&mut self, _: &Box<RpcCall>){
        unimplemented!()
    }

    fn prepare(&mut self){
        unimplemented!()
    }

    fn update(&mut self){
        unimplemented!()
    }

    fn is_done(&self) -> bool{
        self.inflights.is_empty() || self.is_finished()
    }
}

impl LookupTask for NodeLookupTask {
    fn target(&self) -> &Id {
        &self.target
    }

    fn candidate_node(&self) -> &CandidateNode {
        unimplemented!()
    }

    fn closest_set(&self) -> ClosestSet {
        unimplemented!()
    }

    fn add_candidates(&mut self, _: &[Node]) {
        unimplemented!()
    }

    fn remove_candidate(&mut self, _: &Id) {
        unimplemented!()
    }

    fn next_candidate(&self) -> Box<CandidateNode> {
        unimplemented!()
    }

    fn add_closest(&mut self,_: &Box<CandidateNode>) {
        unimplemented!()
    }
}

impl fmt::Display for NodeLookupTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}[{}] DHT:{}, state:{}",
            self.taskid,
            self.name.as_str(),
            "ipv4<TODO>",
            self.state
        )?;
        Ok(())
    }
}
