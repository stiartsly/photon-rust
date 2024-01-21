use std::time::SystemTime;

use crate::rpccall::RpcCall;
use crate::msg::message::Message;

#[allow(dead_code)]
pub(crate) enum State {
    INITIAL,
    QUEUED,
    RUNNING,
    FINISHED,
    CANCELED
}

pub(crate) trait Task {
    fn taskid(&self) -> i32;
    fn name(&self) -> &str;
    fn state(&self) -> &State;

    fn nested(&self) -> &Box<dyn Task>;

    fn is_canceled(&self) -> bool;
    fn is_finished(&self) -> bool;

    fn started_time(&self) -> &SystemTime;
    fn finished_time(&self) -> &SystemTime;

    fn age(&self) -> u64;

    fn with_name(&mut self, _: &str);
    fn set_nested(&mut self, _: Box<dyn Task>);

    //fn add_listener<F>(&mut self, f: F) where F: FnMut(&Box<dyn Task>);
    //fn remove_listener<F>(&mut self, f: F) where F: FnMut(&Box<dyn Task>);

    fn start(&self);
    fn cancel(&self);

    fn call_sent(&mut self, _: &Box<RpcCall>);
    fn call_response(&mut self, _: &Box<RpcCall>, _: &dyn Message);
    fn call_error(&mut self, _: &Box<RpcCall>);
    fn call_timeout(&mut self, _: &Box<RpcCall>);

    fn prepare(&mut self);
    fn update(&mut self);

    fn is_done(&self) -> bool;
}
