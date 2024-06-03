use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;

use super::task::{Task, TaskData};
use crate::{
    kbucket::KBucket,
    kbucket_entry::KBucketEntry,
    rpccall::RpcCall,
    msg::msg::Msg
};

#[allow(dead_code)]
pub(crate) struct PingRefreshTask {
    bucket: Rc<KBucket>,
    todo: LinkedList<Box<KBucketEntry>>,

    check_all: bool,
    probe_cache: bool,
    remove_on_timeout: bool,
}

impl Task for PingRefreshTask {
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
