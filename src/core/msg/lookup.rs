
use crate::id::Id;
use crate::nodeinfo::NodeInfo;

 pub(crate) trait Lookup {
    fn target(&self) -> &Id;
    fn want4(&self) -> bool;
    fn want6(&self) -> bool;
    fn want_token(&self) -> bool;
 }

 pub(crate) trait LookupBuilder<'a> {
    fn with_target(&mut self, _: &'a Id) -> &mut Self;
    fn with_want4(&mut self) -> &mut Self;
    fn with_want6(&mut self) -> &mut Self;
    fn with_token(&mut self) -> &mut Self;
 }

 pub(crate) trait LookupResult {
    fn nodes4(&self) -> &[NodeInfo];
    fn nodes6(&self) -> &[NodeInfo];
    fn token(&self) -> i32;
 }

 pub(crate) trait LookupResultBuilder {
    fn populate_closest_nodes4<F>(&mut self, _:bool, f: F) -> &mut Self
    where F: Fn() -> Vec<NodeInfo>;
    fn populate_closest_nodes6<F>(&mut self, _:bool, f: F) -> &mut Self
    where F: Fn() -> Vec<NodeInfo>;
    fn populate_token<F>(&mut self, _: bool, f: F) -> &mut Self
    where F: Fn() -> i32;
 }
