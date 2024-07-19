use std::rc::Rc;
use std::cell::RefCell;

use crate::{
    is_bogon_addr,
    constants,
    id::Id,
    node_info::{NodeInfo, Reachable},
    rpccall::RpcCall,
    dht::DHT,
};

use crate::msg::{
    lookup_rsp::{Msg as LookupResponse},
};

use super::{
    candidate_node::CandidateNode,
    closest_set::ClosestSet,
    closest_candidates::ClosestCandidates,
};

pub(crate) struct LookupTaskData {
    target: Rc<Id>,
    closest_set: ClosestSet,
    closest_candidates: ClosestCandidates,
}

impl LookupTaskData {
    pub(crate) fn new(target: &Rc<Id>) -> Self {
        Self {
            target: target.clone(),
            closest_set: ClosestSet::new(
                target, constants::MAX_ENTRIES_PER_BUCKET
            ),
            closest_candidates: ClosestCandidates::new(
                target, 3 * constants::MAX_ENTRIES_PER_BUCKET,
            )
        }
    }
}

pub(crate) trait LookupTask {
    fn data(&self) -> &LookupTaskData;
    fn data_mut(&mut self) -> &mut LookupTaskData;
    fn dht(&self) -> Rc<RefCell<DHT>>;

    fn target(&self) -> Rc<Id> {
        self.data().target.clone()
    }

    fn candidate(&self, id: &Id) -> Option<Rc<RefCell<CandidateNode>>>  {
        self.data().closest_candidates.get(id)
    }

    fn next_candidate(&mut self) -> Option<Rc<RefCell<CandidateNode>>> {
        self.data_mut().closest_candidates.next()
    }

    fn add_candidates(&mut self, nodes: &[Rc<NodeInfo>]) {
        let mut candidates = Vec::new();

        let dht = self.dht();
        let binding_dht = dht.borrow();

        for item in nodes.iter() {
            if is_bogon_addr!(item.socket_addr()) ||
                binding_dht.node_id() == item.id() ||
                binding_dht.socket_addr() == item.socket_addr() ||
                self.data().closest_set.contains(item.id()) {
                continue;
            }
            candidates.push(item.clone());
        }

        if !candidates.is_empty() {
            self.data_mut().closest_candidates.add(candidates.as_slice())
        }
    }

    fn remove_candidate(&mut self, id: &Id) -> Option<Rc<RefCell<CandidateNode>>> {
        self.data_mut().closest_candidates.remove(id)
    }

    fn closest_set(&self) -> &ClosestSet {
        &self.data().closest_set
    }

    fn add_closest(&mut self, candidate_node: Rc<RefCell<CandidateNode>>) {
        self.data_mut().closest_set.add(candidate_node)
    }

    fn is_done(&self) -> bool {
        let data = self.data();
        data.closest_candidates.len() == 0 ||
            (data.closest_set.is_eligible() &&
                data.target.three_way_compare(
                    &data.closest_set.tail(), &data.closest_candidates.head()).is_le())
    }

    fn call_error(&mut self, call: &RpcCall) {
        _ = self.remove_candidate(call.target_nodeid())
    }

    fn call_timeout(&mut self, call: &RpcCall) {
        let mut candidate = Box::new(CandidateNode::new(&call.target(), false));
        if candidate.unreachable() {
            self.remove_candidate(candidate.nodeid());
            return;
        }
        // Clear the sent time-stamp and make it available again for the next retry
        candidate.clear_sent()
    }

    fn call_responsed(&mut self, call: &RpcCall, rsp: &dyn LookupResponse) {
        if let Some(cn) = self.remove_candidate(call.target_nodeid()) {
            cn.borrow_mut().set_replied();
            cn.borrow_mut().set_token(rsp.token());
            self.add_closest(cn);
        }
    }
}
