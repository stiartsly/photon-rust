use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use log::{warn, error};

use crate::{
    constants,
    id::Id,
    node_info::NodeInfo,
    value::Value,
    dht::DHT,
    rpccall::RpcCall,
    kclosest_nodes::KClosestNodes,
};

use crate::msg::{
    find_value_req,
    find_value_rsp,
    msg::{self, Msg},
    lookup_req::{Msg as LookupRequest},
    lookup_rsp::{Msg as LookupResponse},
};

use super::{
    task::{Task, TaskData},
    lookup_task::{LookupTask, LookupTaskData},
};

pub(crate) struct ValueLookupTask {
    base_data: TaskData,
    lookup_data: LookupTaskData,

    dht: Rc<RefCell<DHT>>,

    ni: NodeInfo,

    expected_seq: i32,
    result_fn: Box<dyn FnMut(&mut Box<dyn Task>, &mut Option<Box<Value>>)>,
}

impl ValueLookupTask {
    pub(crate) fn new(target: &Id, dht: Rc<RefCell<DHT>>) -> Self {
        Self {
            base_data: TaskData::new(),
            lookup_data: LookupTaskData::new(target),
            ni: NodeInfo::new(dht.borrow().node_id(), dht.borrow().socket_addr()),
            dht: Rc::clone(&dht),
            expected_seq: -1,
            result_fn: Box::new(|_,_|{}),
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(&mut Box<dyn Task>, &mut Option<Box<Value>>) + 'static,
    {
        self.result_fn = Box::new(f);
    }

    //fn with_seq(&mut self, seq: i32) {
    //    self.expected_seq = seq;
    //}
}

impl LookupTask for ValueLookupTask {
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

impl Task for ValueLookupTask {
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
        let mut kclosest_nodes = KClosestNodes::with_filter(
            LookupTask::target(self),
            self.dht.borrow().routing_table(),
            constants::MAX_ENTRIES_PER_BUCKET *2,
            move |_| true
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

            let mut req = find_value_req::Message::new();
            req.with_target(self.target().clone());
            req.with_want4(true);
            req.with_want6(false);

            if self.expected_seq >= 0 {
                req.with_seq(self.expected_seq);
            }

            let cloned_next = Rc::clone(&next);
            let cloned_req = Rc::new(RefCell::new(req));
            if let Err(err) = self.send_call(next, cloned_req, Box::new(move|_| {
                cloned_next.borrow_mut().set_sent();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }

    fn call_responsed(&mut self, call: &RpcCall, rsp: Rc<RefCell<dyn Msg>>) {
        let binding = rsp.borrow();
        if let Some(downcasted) = binding.as_any().downcast_ref::<find_value_rsp::Message>() {
            LookupTask::call_responsed(self, call, downcasted);

            if !call.matches_id()||
                binding.kind() != msg::Kind::Response ||
                binding.method() != msg::Method::FindValue {
                return;
            }

            if let Some(value) = downcasted.value() {
                let id = value.id();
                if &id == LookupTask::target(self) {
                    warn!("Responsed value id {} mismatched with expected {}", id, LookupTask::target(self));
                    return;
                }

                if !value.is_valid() {
                    warn!("Responsed value {} is invalid, signature mismatch", id);
                    return;
                }

                if self.expected_seq >=0 && value.sequence_number() < self.expected_seq {
                    warn!("Responsed value {} is outdated, sequence {}, expected {}",
                        id, value.sequence_number(), self.expected_seq);
                    return;
                }

                //(self.result_fn)(self.clone(), value)
            } else {
                if let Some(nodes) = LookupResponse::nodes4(downcasted) {
                    if !nodes.is_empty() {
                        self.add_candidates(nodes);
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

impl fmt::Display for ValueLookupTask {
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
