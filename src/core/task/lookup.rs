use crate::id::Id;
use super::candidate_node::CandidateNode;
use super::closest_set::ClosestSet;
use super::task::Task;

pub(crate) trait LookupTask: Task {
    fn target(&self) -> &Id;
    fn candidate_node(&self) -> &CandidateNode;
    fn closest_set(&self) -> ClosestSet;
}
