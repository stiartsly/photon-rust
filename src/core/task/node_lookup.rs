use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use log::error;

use crate::{
    constants,
    Id,
    NodeInfo,
    rpccall::RpcCall,
    dht::DHT,
    kclosest_nodes::KClosestNodes,
};

use crate::msg::{
    find_node_req,
    find_node_rsp,
    msg::{Method, Kind, Msg},
    lookup_req::{Msg as LookupRequest},
    lookup_rsp::{Msg as LookupResponse},
};

use super::{
    task::{Task, TaskData},
    lookup_task::{LookupTask, LookupTaskData},
};

pub(crate) struct NodeLookupTask {
    base_data: TaskData,
    lookup_data: LookupTaskData,

    bootstrap: bool,
    want_token: bool,
    result_fn: Box<dyn FnMut(Rc<RefCell<dyn Task>>, Option<Rc<NodeInfo>>)>
}

impl NodeLookupTask {
    pub(crate) fn new(target: &Rc<Id>, dht: Rc<RefCell<DHT>>) -> Self {
        Self {
            base_data: TaskData::new(dht),
            lookup_data: LookupTaskData::new(target),
            bootstrap: false,
            want_token: false,
            result_fn: Box::new(|_,_|{})
        }
    }

    pub(crate) fn set_bootstrap(&mut self, bootstrap: bool) {
        self.bootstrap = bootstrap
    }

    pub(crate) fn set_want_token(&mut self, token: bool) {
        self.want_token = token
    }

    pub(crate) fn inject_candidates(&mut self, nodes: &[Rc<NodeInfo>]) {
        self.add_candidates(nodes)
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(Rc<RefCell<dyn Task>>, Option<Rc<NodeInfo>>) + 'static {
        self.result_fn = Box::new(f)
    }
}

impl LookupTask for NodeLookupTask {
    fn data(&self) -> &LookupTaskData {
        &self.lookup_data
    }

    fn data_mut(&mut self) -> &mut LookupTaskData {
        &mut self.lookup_data
    }

    fn dht(&self) -> Rc<RefCell<DHT>> {
        Task::data(self).dht()
    }
}

impl Task for NodeLookupTask {
    fn data(&self) -> &TaskData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut TaskData {
        &mut self.base_data
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn prepare(&mut self) {
        // if we're bootstrapping start from the bucket that has the greatest
        // possible distance from ourselves so we discover new things along
        // the (longer) path.
        let target = match self.bootstrap {
            true => Rc::new(self.target().distance(&Id::max())),
            false => self.target().clone()
        };

        // delay the filling of the todo list until we actually start the task
        let mut kclosest_nodes = KClosestNodes::with_filter(
            target,
            Task::data(self).rt(),
            constants::MAX_ENTRIES_PER_BUCKET *2,
            move |e| e.is_eligible_for_nodes_list()
        );
        kclosest_nodes.fill(false);
        let nodes = kclosest_nodes.as_nodes();
        self.add_candidates(&nodes);
    }

    fn update(&mut self) {
        while self.can_request() {
            let next = match LookupTask::next_candidate(self) {
                Some(next) => next.clone(),
                None => break ,
            };

            let mut msg = find_node_req::Message::new();
            msg.with_target(self.target());
            msg.with_want4(true);
            msg.with_want6(false);

            let msg = Rc::new(RefCell::new(msg));
            let ni  = next.borrow().ni();
            let cloned_next = next.clone();

            let _ = self.send_call(ni, msg, Box::new(move|_| {
                cloned_next.borrow_mut().set_sent();
            })).map_err(|e| {
                error!("Error on sending 'findNode' message: {:?}", e);
            });
        }
    }

    fn call_responsed(&mut self, call: &RpcCall, msg: Rc<RefCell<dyn Msg>>) {
        if let Some(msg) = msg.borrow().as_any().downcast_ref::<find_node_rsp::Message>() {
            LookupTask::call_responsed(self, call, msg);

            if !call.matches_id()||
                msg.kind() != Kind::Response ||
                msg.method() != Method::FindNode {
                return;
            }

            if let Some(nodes) = msg.nodes4() { // TODO:
                if !nodes.is_empty() {
                    self.add_candidates(nodes);
                }

                for item in nodes.iter() {
                    if item.id() == self.target().as_ref() {
                        (self.result_fn)(self.base_data.task(), Some(item.clone()));
                    }
                }
            }
        }
    }

    fn call_error(&mut self, call: &RpcCall) {
        LookupTask::call_error(self, call)
    }

    fn call_timeout(&mut self, call: &RpcCall) {
        LookupTask::call_timeout(self, call)
    }
}

impl fmt::Display for NodeLookupTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "#{}[{}] DHT:{}, state:{}",
            self.taskid(),
            self.name(),
            "ipv4",
            self.state()
        )?;
        Ok(())
    }
}
