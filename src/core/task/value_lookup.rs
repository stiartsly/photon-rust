use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use log::{warn, error};

use crate::{
    constants,
    Id,
    Value,
    dht::DHT,
    rpccall::RpcCall,
    kclosest_nodes::KClosestNodes,
};

use crate::msg::{
    find_value_req,
    find_value_rsp,
    msg::{Method, Kind, Msg},
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

    expected_seq: i32,
    result_fn: Box<dyn FnMut(Rc<RefCell<dyn Task>>, Option<Rc<Value>>)>,
}

impl ValueLookupTask {
    pub(crate) fn new(dht: Rc<RefCell<DHT>>, target: &Rc<Id>) -> Self {
        Self {
            base_data: TaskData::new(dht),
            lookup_data: LookupTaskData::new(target),
            expected_seq: -1,
            result_fn: Box::new(|_,_|{}),
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(Rc<RefCell<dyn Task>>, Option<Rc<Value>>) + 'static,
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

    fn dht(&self) -> Rc<RefCell<DHT>> {
        Task::data(self).dht()
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
            Task::data(self).dht(),
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
                Some(next) => next.clone(),
                None => break ,
            };

            let mut msg = find_value_req::Message::new();
            msg.with_target(self.target().clone());
            msg.with_want4(true);
            msg.with_want6(false);

            if self.expected_seq >= 0 {
                msg.with_seq(self.expected_seq);
            }

            let msg = Rc::new(RefCell::new(msg));
            let cloned_next = next.clone();
            let ni = next.borrow().ni();

            let _ = self.send_call(ni, msg, Box::new(move|_| {
                cloned_next.borrow_mut().set_sent();
            })).map_err(|e| {
               error!("Error on sending 'findNode' message: {:?}", e);
            });
        }
    }

    fn call_responsed(&mut self, call: &RpcCall, msg: Rc<RefCell<dyn Msg>>) {
        if let Some(msg) = msg.borrow().as_any().downcast_ref::<find_value_rsp::Message>() {
            LookupTask::call_responsed(self, call, msg);

            if !call.matches_id()||
                msg.kind() != Kind::Response ||
                msg.method() != Method::FindValue {
                return;
            }

            if let Some(value) = msg.value() {
                let id = value.id();
                if &id == LookupTask::target(self).as_ref() {
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

                (self.result_fn)(self.base_data.task(), Some(value.clone()));
            } else {
                if let Some(nodes) = LookupResponse::nodes4(msg) {
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
