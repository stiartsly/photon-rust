use std::rc::Rc;
use std::any::Any;
use std::collections::LinkedList;
use std::time::SystemTime;
use std::boxed::Box;

use crate::{
    kbucket::KBucket,
    kbucket_entry::KBucketEntry,
    rpccall::RpcCall,
    msg::msg::Msg
};
use super::task::{Task, State};

#[allow(dead_code)]
pub(crate) struct PingRefreshTask {
    bucket: Rc<KBucket>,
    todo: LinkedList<Box<KBucketEntry>>,

    check_all: bool,
    probe_cache: bool,
    remove_on_timeout: bool,
}

impl Task for PingRefreshTask {
    fn taskid(&self) -> i32 {
        unimplemented!()
    }

    fn name(&self) -> &str{
        unimplemented!()
    }

    fn with_name(&mut self, _: &str) {
        unimplemented!()
    }

    fn state(&self) -> State{
        unimplemented!()
    }

    fn nested(&self) -> &Box<dyn Task> {
        unimplemented!()
    }

    fn is_canceled(&self) -> bool{
        unimplemented!()
    }

    fn is_finished(&self) -> bool{
        unimplemented!()
    }

    fn started_time(&self) -> &SystemTime{
        unimplemented!()
    }

    fn finished_time(&self) -> &SystemTime{
        unimplemented!()
    }

    fn age(&self) -> u128 {
        unimplemented!()
    }

    fn set_nested(&mut self, _: Box<dyn Task>) {
        unimplemented!()
    }

    fn start(&mut self){
        unimplemented!()
    }

    fn cancel(&mut self){
        unimplemented!()
    }

    fn call_sent(&mut self, _: &Box<RpcCall>){
        unimplemented!()
    }

    fn call_responsed(&mut self, _: &Box<RpcCall>, _: &Box<dyn Msg>){
        unimplemented!()
    }

    fn call_error(&mut self, _: &Box<RpcCall>){
        unimplemented!()
    }

    fn call_timeout(&mut self, _: &Box<RpcCall>){
        unimplemented!()
    }

    fn prepare(&mut self){
        unimplemented!()
    }

    fn update(&mut self){
        unimplemented!()
    }

    fn is_done(&self) -> bool{
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
