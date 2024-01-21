use std::rc::Rc;
use std::collections::LinkedList;
use std::time::SystemTime;
use std::boxed::Box;
use super::task::{Task, State};

use crate::kbucket::KBucket;
use crate::kbucket_entry::KBucketEntry;
use crate::rpccall::RpcCall;
use crate::msg::message::Message;

#[allow(dead_code)]
pub(crate) struct PingRefreshTask<'a> {
    bucket: Rc<KBucket>,
    todo: LinkedList<Box<KBucketEntry>>,

    check_all: bool,
    probe_cache: bool,
    remove_on_timeout: bool,
}

impl<'a> PingRefreshTask<'a> {
    fn taskid(&self) -> i32 {
        unimplemented!()
    }

    fn name(&self) -> &str{
        unimplemented!()
    }

    fn state(&self) -> &State{
        unimplemented!()
    }

    fn nested_task(&self) -> &Box<dyn Task>{
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

    fn age(&self) -> u64{
        unimplemented!()
    }

    fn with_name(&mut self, _: &'a str){
        unimplemented!()
    }

    fn set_nested_task(&mut self, _: Box<dyn Task>){
        unimplemented!()
    }

    fn add_listener<F>(&mut self, f: F) where F: FnMut(&Box<dyn Task>){
        unimplemented!()
    }

    fn remove_listener<F>(&mut self, f: F) where F: FnMut(&Box<dyn Task>){
        unimplemented!()
    }

    fn start(&self){
        unimplemented!()
    }

    fn cancel(&self){
        unimplemented!()
    }

    fn call_sent(&mut self, _: &Box<RpcCall>){
        unimplemented!()
    }

    fn call_response(&mut self, _: &Box<RpcCall>, _: impl Message){
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
}
