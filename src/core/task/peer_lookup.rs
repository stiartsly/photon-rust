use std::any::Any;
use std::rc::Rc;
use std::net::SocketAddr;
use std::cell::RefCell;
use log::error;

use crate::{
    constants,
    id::Id,
    peer::Peer,
    node_info::NodeInfo,
    rpccall::RpcCall,
    routing_table::RoutingTable,
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

#[allow(dead_code)]
pub(crate) struct PeerLookupTask {
    base_data: TaskData,
    lookup_data: LookupTaskData,

    routing_table: Rc<RefCell<RoutingTable>>,
    ni: NodeInfo,

    result_fn: Box<dyn FnMut(&mut Box<dyn Task>, &mut Vec<Box<Peer>>)>,
}

impl PeerLookupTask {
    pub(crate) fn new(target: &Id,
        routing_table: Rc<RefCell<RoutingTable>>) -> Self {
        Self {
            base_data: TaskData::new(),
            lookup_data: LookupTaskData::new(target),
            ni: NodeInfo::new(routing_table.borrow().node_id(), routing_table.borrow().node_addr()),
            routing_table: Rc::clone(&routing_table),
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

    fn node_id(&self) -> &Id {
        self.ni.id()
    }
    fn node_address(&self) -> &SocketAddr {
        self.ni.socket_addr()
    }
}

impl Task for PeerLookupTask {
    fn data(&self) -> &TaskData {
        unimplemented!()
    }
    fn data_mut(&mut self) -> &mut TaskData {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn prepare(&mut self) {
        let mut kclosest_nodes = KClosestNodes::new_with_filter(
            LookupTask::target(self),
            Rc::clone(&self.routing_table),
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

            let req = Rc::new(RefCell::new(find_peer_req::Message::new()));
            req.borrow_mut().with_target(self.target().clone());
            req.borrow_mut().with_want4(true);
            req.borrow_mut().with_want6(false);

            let cloned = Rc::clone(&next);
            if let Err(err) = self.send_call(next, req, Box::new(move|_| {
                cloned.borrow_mut().set_sent();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }

    fn call_sent(&mut self, _: &RpcCall) {}

    fn call_responsed(&mut self, call: &RpcCall, rsp: Rc<RefCell<dyn Msg>>) {
        let binding = rsp.borrow();
        if let Some(downcasted) = binding.as_any().downcast_ref::<find_peer_rsp::Message>() {
            LookupTask::call_responsed(self, call, downcasted);

            if !call.matches_id()||
                binding.kind() != msg::Kind::Response ||
                binding.method() != msg::Method::FindNode {
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
