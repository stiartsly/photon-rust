use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeStatus {
    Stopped,
    Initializing,
    Running,
}

impl fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeStatus::Stopped => write!(f, "Stopped"),
            NodeStatus::Initializing => write!(f, "Initializing"),
            NodeStatus::Running => write!(f, "Running"),
        }
    }
}
