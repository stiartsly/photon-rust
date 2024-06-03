use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;

use super::{
    closest_set::ClosestSet,
    task::{Task, TaskData},
};
use crate::{
    kbucket::KBucket,
    kbucket_entry::KBucketEntry,
    value::Value,
    rpccall::RpcCall,
    msg::msg::Msg,
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
        unimplemented!()
    }

    fn update(&mut self) {
        unimplemented!()
    }

    fn call_sent(&mut self, _: &RpcCall) {
        unimplemented!()
    }

    fn call_responsed(&mut self, _: &RpcCall, _: Rc<RefCell<dyn Msg>>) {
        unimplemented!()
    }

    fn call_error(&mut self, _: &RpcCall) {
        unimplemented!()
    }

    fn call_timeout(&mut self, _: &RpcCall) {
        unimplemented!()
    }
}
