use std::any::Any;
use std::fmt;

use super::{
    candidate_node::CandidateNode,
    closest_set::ClosestSet
};

use crate::{
    id::Id,
    node_info::NodeInfo,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Initial,
    Queued,
    Running,
    Finished,
    Canceled,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            State::Initial => "INITIAL",
            State::Queued => "QUEUED",
            State::Running => "RUNNING",
            State::Finished => "FINISHED",
            State::Canceled => "CANCELED",
        };
        write!(f, "{}", str)?;
        Ok(())
    }
}

pub(crate) trait Task {
    fn taskid(&self) -> i32;
    fn name(&self) -> &str;
    fn set_name(&mut self, _: &str);
    fn state(&self) -> State;
    fn set_state(&mut self, _:&[State], _: State) -> bool { true }
    fn nested(&self) -> &Box<dyn Task> { panic!() }
    fn set_nested(&mut self, _: Box<dyn Task>) { panic!() }

    fn add_listener(&mut self, _: Box<dyn FnOnce(&dyn Task)>) { panic!()}

    fn start(&mut self);
    fn cancel(&mut self);

    fn is_canceled(&self) -> bool {true}
    fn is_finished(&self) -> bool {true}

    fn as_any(&self) -> &dyn Any;
}

pub(crate) trait Lookup: Task {
    fn target(&self) -> &Id;
    fn candidate_node(&self, id: &Id) -> Option<&Box<CandidateNode>>;
    fn closest_set(&self) -> &ClosestSet;

    fn add_candidates(&mut self, _: &[NodeInfo]) { panic!()}
    fn remove_candidate(&mut self, _: &Id) { panic!() }
    fn next_candidate(&self) -> Option<&Box<CandidateNode>> { panic!() }
    fn add_closest(&mut self, _: Box<CandidateNode>) { panic!() }
}
