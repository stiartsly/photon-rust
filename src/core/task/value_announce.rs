use std::any::Any;
use std::boxed::Box;
use std::collections::LinkedList;
use std::rc::Rc;

use super::{
    closest_set::ClosestSet,
    task::{State, Task},
};
use crate::{
    kbucket::KBucket, kbucket_entry::KBucketEntry, value::Value,
};

#[allow(dead_code)]
pub(crate) struct ValueAnnounceTask {
    bucket: Rc<KBucket>,
    todo: LinkedList<Box<KBucketEntry>>,

    check_all: bool,
    probe_cache: bool,
    remove_on_timeout: bool,
}

#[allow(dead_code)]
impl ValueAnnounceTask {
    pub(crate) fn new(_: &ClosestSet, _: &Value) -> Self {
        unimplemented!()
    }
}

impl Task for ValueAnnounceTask {
    fn taskid(&self) -> i32 {
        unimplemented!()
    }

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn set_name(&mut self, _: &str) {
        unimplemented!()
    }

    fn state(&self) -> State {
        unimplemented!()
    }

    fn nested(&self) -> &Box<dyn Task> {
        unimplemented!()
    }

    fn is_canceled(&self) -> bool {
        unimplemented!()
    }

    fn is_finished(&self) -> bool {
        unimplemented!()
    }

    fn set_nested(&mut self, _: Box<dyn Task>) {
        unimplemented!()
    }

    fn start(&mut self) {
        unimplemented!()
    }

    fn cancel(&mut self) {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
