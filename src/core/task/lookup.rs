use crate::id::Id;
use crate::node::Node;
use super::candidate_node::CandidateNode;
use super::closest_set::ClosestSet;
use super::task::Task;

pub(crate) trait LookupTask: Task {
    fn target(&self) -> &Id;
    fn candidate_node(&self) -> &CandidateNode;
    fn closest_set(&self) -> ClosestSet;

    fn add_candidates(&mut self, _: &[Node]) {
        unimplemented!()
    }

    fn remove_candidate(&mut self, _: &Id) {
        unimplemented!()
    }

    fn next_candidate(&self) -> Box<CandidateNode> {
        unimplemented!()
    }

    fn add_closest(&mut self,_: &Box<CandidateNode>) {
        unimplemented!()
    }
}
