use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use log::error;

use crate::{
    constants,
    id::Id,
    peer::Peer,
    dht::DHT,
    rpccall::RpcCall,
    kclosest_nodes::KClosestNodes,
};

use crate::msg::{
    find_peer_req,
    find_peer_rsp,
    msg::{self, Msg},
    lookup_req::{Msg as LookupRequest},
};

use super::{
    task::{Task, TaskData},
    lookup_task::{LookupTask, LookupTaskData},
};

pub(crate) struct PeerLookupTask {
    base_data: TaskData,
    lookup_data: LookupTaskData,

    result_fn: Box<dyn FnMut(&mut Box<dyn Task>, &mut Vec<Box<Peer>>)>,
}

impl PeerLookupTask {
    pub(crate) fn new(target: &Rc<Id>, dht: Rc<RefCell<DHT>>) -> Self {
        Self {
            base_data: TaskData::new(dht),
            lookup_data: LookupTaskData::new(target),
            result_fn: Box::new(|_,_|{}),
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where
        F: FnMut(&mut Box<dyn Task>, &mut Vec<Box<Peer>>) + 'static,
    {
        self.result_fn = Box::new(f);
    }
}

impl LookupTask for PeerLookupTask {
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

impl Task for PeerLookupTask {
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
            Task::data(self).rt(),
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

            let mut req = find_peer_req::Message::new();
            req.with_target(self.target().clone());
            req.with_want4(true);
            req.with_want6(false);

            let cloned_text = Rc::clone(&next);
            let cloned_req = Rc::new(RefCell::new(req));
            if let Err(err) = self.send_call(next, cloned_req, Box::new(move|_| {
                cloned_text.borrow_mut().set_sent();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }

    fn call_responsed(&mut self, call: &RpcCall, rsp: Rc<RefCell<dyn Msg>>) {
        let binding = rsp.borrow();
        if let Some(downcasted) = binding.as_any().downcast_ref::<find_peer_rsp::Message>() {
            LookupTask::call_responsed(self, call, downcasted);

            if !call.matches_id()||
                binding.kind() != msg::Kind::Response ||
                binding.method() != msg::Method::FindPeer {
                return;
            }

            if downcasted.has_peers() {
                for peer in downcasted.peers() {
                    if !peer.is_valid() {
                        error!("Response include invalid peer, signature mismatched.");
                        return; // ignored.
                    }
                }
            }
            //(self.result_fn)(self.clone(), downcased.peers())
        }
    }

    fn call_error(&mut self, call: &RpcCall) {
        LookupTask::call_error(self, call)
    }

    fn call_timeout(&mut self, call: &RpcCall) {
        LookupTask::call_timeout(self, call)
    }
}

impl fmt::Display for PeerLookupTask {
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
