use std::fmt;
use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;
use log::{error, debug};

use crate::{
    kbucket::KBucket,
    kbucket_entry::KBucketEntry,
    rpccall::RpcCall,
    dht::DHT,
};

use crate::msg::{
    ping_req,
};

use super::task::{
    Task,
    TaskData
};

#[allow(dead_code)]
pub(crate) struct PingRefreshTask {
    base_data: TaskData,

    bucket: Option<Rc<KBucket>>,
    todo: Rc<RefCell<LinkedList<Rc<RefCell<KBucketEntry>>>>>,

    check_all: bool,
    // probe_cache: bool,
    remove_on_timeout: bool,
}

#[allow(dead_code)]
impl PingRefreshTask {
    pub(crate) fn new(dht: Rc<RefCell<DHT>>) -> Self {
        Self {
            base_data: TaskData::new(dht),
            bucket: None,
            todo: Rc::new(RefCell::new(LinkedList::new())),
            check_all: false,
            remove_on_timeout: false,
        }
    }
}

impl Task for PingRefreshTask {
    fn data(&self) -> &TaskData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut TaskData {
        &mut self.base_data
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self) {
        while self.can_request() {
            let candidate_node = {
                let cn = match self.todo.borrow().front() {
                    Some(cn) => cn.clone(),
                    None => break,
                };
                if !self.check_all && !cn.borrow().needs_ping() {
                    self.todo.borrow_mut().pop_front();
                }
                cn
            };

            let msg = Rc::new(RefCell::new(ping_req::Message::new()));
            let ni  = candidate_node.borrow().ni();
            let cloned_todo = self.todo.clone();

            let _ = self.send_call(ni, msg, Box::new(move|_| {
                cloned_todo.borrow_mut().pop_front();
            })).map_err(|e| {
               error!("Error on sending 'pingRequest' message: {:?}", e);
            });
        }
    }

    fn call_timeout(&mut self, call: &RpcCall) {
        if self.remove_on_timeout {
            return;
        }

        // CAUSION:
        // Should not use the original bucket object,
        // because the routing table is dynamic, maybe already changed.
        debug!("Removing invalid entry from routingtable");
        Task::data(self).rt().borrow_mut().remove(call.target_id());
    }
}

impl fmt::Display for PingRefreshTask {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
