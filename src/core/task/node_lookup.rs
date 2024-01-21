//use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::id::Id;
use crate::node::Node;
//use crate::dht::DHT;
use crate::rpccall::RpcCall;
use crate::msg::msg::Msg;
use super::task::{Task, State};
use super::lookup::LookupTask;
use super::candidate_node::CandidateNode;
use super::closest_set::ClosestSet;
use super::closest_candidates::ClosestCandidates;

use log::{debug};

#[allow(dead_code)]
pub(crate) struct NodeLookupTask<'a> {
    //dht: Rc<&'a DHT>,
    taskid: i32,
    name: Option<String>,
    state: State,

    // nested: *const libc::c_void,

    started_time: SystemTime,
    finished_time: SystemTime,

    inflights: HashMap<usize, Box<RpcCall>>,
    listeners: Vec<Box<dyn FnMut(&Box<dyn Task>)>>,

    // Lookup
    target: Option<Id>,
    closest_set: Option<ClosestSet<'a>>,
    closest_candidates: Option<ClosestCandidates>,

    // NodeLookup
    bootstrap: bool,
    want_token: bool,
    result_fn: Option<Box<dyn FnMut(Option<Box<Node>>, &mut Box<dyn Task>)>>,
}

#[allow(dead_code)]
impl<'a> NodeLookupTask<'a> {
    pub(crate) fn new(id: &Id) -> Self {
        NodeLookupTask { //dht,
            taskid: 0,
            name: None,
            state: State::INITIAL,
            started_time: SystemTime::UNIX_EPOCH,
            finished_time: SystemTime::UNIX_EPOCH,
            inflights: HashMap::new(),
            listeners: Vec::new(),

            target: Some(id.clone()),
            closest_set: None,
            closest_candidates: None,

            bootstrap: false,
            want_token: false,
            result_fn: None,
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(Option<Box<Node>>, &mut Box<dyn Task>) + 'static {
        self.result_fn = Some(Box::new(f));
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
}

impl<'a> Task for NodeLookupTask<'a> {
    fn taskid(&self) -> i32 {
        self.taskid
    }

    fn name(&self) -> &str{
        match self.name.as_ref() {
            Some(n) => &n,
            None => "task"
        }
    }

    fn state(&self) -> State{
        self.state
    }

    fn nested(&self) -> &Box<dyn Task> {
        unimplemented!()
    }

    fn is_canceled(&self) -> bool{
        self.state == State::CANCELED
    }

    fn is_finished(&self) -> bool{
        self.state == State::FINISHED
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

    fn with_name(&mut self, name: &str) {
        self.name = Some(name.to_string())
    }

    fn set_nested(&mut self, _: Box<dyn Task>) {
        unimplemented!()
    }

    fn start(&mut self){
        if match self.state {
            State::INITIAL => { self.state = State::RUNNING; true },
            State::QUEUED => { self.state = State::RUNNING; true },
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
            State::INITIAL => { self.state = State::CANCELED; true },
            State::QUEUED => { self.state = State::CANCELED; true },
            State::RUNNING => { self.state = State::CANCELED; true },
            _ => {false}
        } {
            self.finished_time = SystemTime::now();
            debug!("Task canceled: {}", self);

            // self.notify_completion_listeners();
        }
        // if (!!self.nested)
        //    nested.cancel()
    }

    fn call_sent(&mut self, _: &Box<RpcCall>){
        unimplemented!()
    }

    fn call_response(&mut self, _: &Box<RpcCall>, _: &dyn Msg){
        unimplemented!()
    }

    fn call_error(&mut self, _: &Box<RpcCall>){
        unimplemented!()
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

impl<'a> LookupTask for NodeLookupTask<'a> {
    fn target(&self) -> &Id {
        self.target.as_ref().unwrap()
    }

    fn candidate_node(&self) -> &CandidateNode {
        unimplemented!()
    }

    fn closest_set(&self) -> ClosestSet {
        unimplemented!()
    }
}

impl<'a> fmt::Display for NodeLookupTask<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}[{}] DHT:{}, state:{}",
            self.taskid,
            self.name(),
            "ipv4<TODO>",
            self.state
        )?;
        Ok(())
    }
}
