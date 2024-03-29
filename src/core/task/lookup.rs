use super::{candidate_node::CandidateNode, closest_set::ClosestSet, task::Task};
use crate::{id::Id, node_info::NodeInfo};

pub(crate) trait LookupTask: Task {
    fn target(&self) -> &Id;
    fn candidate_node(&self) -> &CandidateNode;
    fn closest_set(&self) -> ClosestSet;

    fn add_candidates(&mut self, _: &[NodeInfo]) {
        unimplemented!()
    }

    fn remove_candidate(&mut self, _: &Id) {
        unimplemented!()
    }

    fn next_candidate(&self) -> Box<CandidateNode> {
        unimplemented!()
    }

    fn add_closest(&mut self, _: &Box<CandidateNode>) {
        unimplemented!()
    }
}
