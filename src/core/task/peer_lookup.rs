use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use super::task::{Task, TaskData};
use crate::{
    id::Id,
    peer::Peer,
    rpccall::RpcCall,
    msg::msg::Msg
};

#[allow(dead_code)]
pub(crate) struct PeerLookupTask {
    //dht: Rc<&'a DHT>,
    //id: Id,
    bootstrap: bool,
    want_token: bool,

    result_fn: Box<dyn FnMut(&mut Box<dyn Task>, &mut Vec<Box<Peer>>)>,
    listeners: Vec<Box<dyn FnMut(&Box<dyn Task>)>>,
}

#[allow(dead_code)]
impl PeerLookupTask {
    pub(crate) fn new(_: &Id) -> Self {
        PeerLookupTask {
            //dht,
            //id: id.clone(),
            bootstrap: false,
            want_token: false,
            result_fn: Box::new(|_, _| {}),
            listeners: Vec::new(),
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where
        F: FnMut(&mut Box<dyn Task>, &mut Vec<Box<Peer>>) + 'static,
    {
        self.result_fn = Box::new(f);
    }

    pub(crate) fn add_listener<F>(&mut self, f: F)
    where
        F: FnMut(&Box<dyn Task>) + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    pub(crate) fn remove_listener<F>(&mut self) {
        self.listeners.pop();
    }
}

impl Task for PeerLookupTask {
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
