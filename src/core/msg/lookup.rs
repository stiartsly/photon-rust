use crate::id::Id;
use crate::node_info::NodeInfo;

pub(crate) trait Filter {
    fn target(&self) -> &Id;
    fn want4(&self) -> bool;
    fn want6(&self) -> bool;
    fn want_token(&self) -> bool;

    fn with_target(&mut self, _: &Id);
    fn with_want4(&mut self);
    fn with_want6(&mut self);
    fn with_token(&mut self);
}

pub(crate) trait Result {
    fn nodes4(&self) -> &[NodeInfo];
    fn nodes6(&self) -> &[NodeInfo];
    fn token(&self) -> i32;

    fn populate_closest_nodes4<F>(&mut self, _: bool, f: F)
    where F: FnOnce() -> Option<Vec<NodeInfo>>;

    fn populate_closest_nodes6<F>(&mut self, _: bool, f: F)
    where F: FnOnce() -> Option<Vec<NodeInfo>>;

    fn populate_token<F>(&mut self, _: bool, f: F)
    where F: FnOnce() -> i32;
}
