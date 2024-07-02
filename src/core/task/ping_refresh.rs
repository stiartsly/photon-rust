use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug};

use crate::{
    kbucket::KBucket,
    kbucket_entry::KBucketEntry,
    rpccall::RpcCall,
    routing_table::RoutingTable,
};

// use crate::msg::{
    // ping_req,
    // msg::Msg,
// };

use super::task::{
    Task,
    TaskData
};

#[allow(dead_code)]
pub(crate) struct PingRefreshTask {
    base_data: TaskData,
    routing_table: Rc<RefCell<RoutingTable>>,

    bucket: Option<Rc<KBucket>>,
    todo: LinkedList<Box<KBucketEntry>>,

    check_all: bool,
    // probe_cache: bool,
    remove_on_timeout: bool,
}

#[allow(dead_code)]
impl PingRefreshTask {
    pub(crate) fn new(routing_table: Rc<RefCell<RoutingTable>>) -> Self {
        Self {
            base_data: TaskData::new(),
            routing_table: Rc::clone(&routing_table),
            bucket: None,
            todo: LinkedList::new(),
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
        /* while self.todo.is_empty() && self.can_request() {
            let candidate_node = match self.todo.pop_front() {
                Some(node) => node,
                None => break,
            };

            if !self.check_all && candidate_node.needs_ping() {
                // Entry already updated during the task running
                self.todo.pop_front();
                continue;
            }

            let req = Rc::new(RefCell::new(ping_req::Message::new()));
            if let Err(err) = self.send_call(candidate_node, req, Box::new(move|_| {
                self.todo.pop_front();
            })) {
               error!("Error on sending 'pingRequest' message: {:?}", err);
            }
        } */
        unimplemented!()
    }

    fn call_timeout(&mut self, call: &RpcCall) {
        if self.remove_on_timeout {
            return;
        }

        // CAUSION:
        // Should not use the original bucket object,
        // because the routing table is dynamic, maybe already changed.
        let node_id = call.target_id();
        debug!("Removing invalid entry from routingtable");
        self.routing_table.borrow_mut().remove(node_id);
    }
}
