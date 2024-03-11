use std::any::Any;

use super::task::{State, Task};
use crate::{id::Id, value::Value};

#[allow(dead_code)]
pub(crate) struct ValueLookupTask {
    //dht: Rc<&'a DHT>,
    //id: Id,
    bootstrap: bool,
    want_token: bool,

    result_fn: Box<dyn FnMut(&mut Box<dyn Task>, Option<Box<Value>>)>,
    listeners: Vec<Box<dyn FnMut(&Box<dyn Task>)>>,
}

#[allow(dead_code)]
impl ValueLookupTask {
    pub(crate) fn new(_: &Id) -> Self {
        ValueLookupTask {
            //dht,
            bootstrap: false,
            want_token: false,
            result_fn: Box::new(|_, _| {}),
            listeners: Vec::new(),
        }
    }

    pub(crate) fn set_result_fn<F>(&mut self, f: F)
    where
        F: FnMut(&mut Box<dyn Task>, Option<Box<Value>>) + 'static,
    {
        self.result_fn = Box::new(f);
    }

    pub(crate) fn add_listener<F>(&mut self, f: F)
    where
        F: FnMut(&Box<dyn Task>) + 'static,
    {
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
