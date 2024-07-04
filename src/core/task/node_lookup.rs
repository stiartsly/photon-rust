use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use log::error;

use crate::{
    constants,
    id::Id,
    node_info::NodeInfo,
    rpccall::RpcCall,
    dht::DHT,
    kclosest_nodes::KClosestNodes,
};

use crate::msg::{
    find_node_req,
    find_node_rsp,
    msg::{self, Msg},
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
    result_fn: Box<dyn FnMut(&mut dyn Task, Option<Box<NodeInfo>>)>,

    dht: Rc<RefCell<DHT>>,
    ni: NodeInfo,
}

impl NodeLookupTask {
    pub(crate) fn new(target: &Id, dht: Rc<RefCell<DHT>>) -> Self {
        Self {
            base_data: TaskData::new(),
            lookup_data: LookupTaskData::new(target),
            bootstrap: false,
            want_token: false,
            result_fn: Box::new(|_,_|{}),
            ni: NodeInfo::new(dht.borrow().node_id(), dht.borrow().socket_addr()),
            dht: Rc::clone(&dht),
        }
    }

    pub(crate) fn set_bootstrap(&mut self, bootstrap: bool) {
        self.bootstrap = bootstrap
    }

    pub(crate) fn set_want_token(&mut self, token: bool) {
        self.want_token = token
    }

    pub(crate) fn inject_candidates(&mut self, nodes: &[NodeInfo]) {
        self.add_candidates(nodes)
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(&mut dyn Task, Option<Box<NodeInfo>>) + 'static {
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

    fn node_id(&self) -> &Id {
        self.ni.id()
    }
    fn node_address(&self) -> &SocketAddr {
        self.ni.socket_addr()
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
            true => self.target().distance(&Id::max()),
            false => self.target().clone()
        };

        // delay the filling of the todo list until we actually start the task
        let mut kclosest_nodes = KClosestNodes::with_filter(
            &target,
            self.dht.borrow().routing_table(),
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
                Some(next) => Rc::clone(&next),
                None => { break },
            };

            let mut req = find_node_req::Message::new();
            req.with_target(self.target().clone());
            req.with_want4(true);
            req.with_want6(false);

            let cloned_req = Rc::new(RefCell::new(req));
            let cloned_next = Rc::clone(&next);
            if let Err(err) = self.send_call(next, cloned_req, Box::new(move|_| {
                cloned_next.borrow_mut().set_sent();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }

    fn call_responsed(&mut self, call: &RpcCall, rsp: Rc<RefCell<dyn Msg>>) {
        let binding = rsp.borrow();
        if let Some(downcasted) = binding.as_any().downcast_ref::<find_node_rsp::Message>() {
            LookupTask::call_responsed(self, call, downcasted);

            if !call.matches_id()||
                binding.kind() != msg::Kind::Response ||
                binding.method() != msg::Method::FindNode {
                return;
            }

            if let Some(nodes) = downcasted.nodes4() { // TODO:
                if !nodes.is_empty() {
                    self.add_candidates(nodes);
                }

                for item in nodes.iter() {
                    if item.id() == self.target() {
                        //(self.result_fn)(self.clone(), None)
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