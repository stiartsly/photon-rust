
//use std::rc::Rc;
use std::time::SystemTime;

use crate::id::Id;
use crate::value::Value;
use crate::rpccall::RpcCall;
use crate::msg::msg::Msg;
use super::task::{Task, State};

#[allow(dead_code)]
pub(crate) struct ValueLookupTask {
    //dht: Rc<&'a DHT>,
    id: Id,

    bootstrap: bool,
    want_token: bool,

    result_fn: Option<Box<dyn FnMut(Option<Box<Value>>, &mut Box<dyn Task>)>>,

    listeners: Vec<Box<dyn FnMut(&Box<dyn Task>)>>,
}

#[allow(dead_code)]
impl ValueLookupTask {
    pub(crate) fn new(id: &Id) -> Self {
        ValueLookupTask {
            //dht,
            id: id.clone(),
            bootstrap: false,
            want_token: false,
            result_fn: None,
            listeners: Vec::new(),
        }
        // TODO:
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where F: FnMut(Option<Box<Value>>, &mut Box<dyn Task>) + 'static {
        self.result_fn = Some(Box::new(f));
    }

    pub(crate) fn add_listener<F>(&mut self, f: F)
    where F: FnMut(&Box<dyn Task>) + 'static {
        self.listeners.push(Box::new(f));
    }

    fn remove_listener<F>(&mut self) {
        self.listeners.pop();
    }
}

impl Task for ValueLookupTask {
    fn taskid(&self) -> i32 {
        unimplemented!()
    }

    fn name(&self) -> &str{
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

    fn with_name(&mut self, _: &str) {
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

    fn call_response(&mut self, _: &Box<RpcCall>, _: &dyn Msg){
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
