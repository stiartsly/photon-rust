use std::fmt;
use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;
use log::error;

use crate::{
    Value,
    dht::DHT,
};

use super::{
    closest_set::ClosestSet,
    candidate_node::CandidateNode,
    task::{Task, TaskData},
};

use crate::msg::{
    store_value_req
};

#[allow(dead_code)]
pub(crate) struct ValueAnnounceTask {
    base_data: TaskData,

    todo: Rc<RefCell<LinkedList<Rc<RefCell<CandidateNode>>>>>,
    peer: Option<Box<Value>>,
}

#[allow(dead_code)]
impl ValueAnnounceTask {
    pub(crate) fn new(dht: Rc<RefCell<DHT>>, closest: &ClosestSet, value: &Value) -> Self {
        let mut todo = LinkedList::new();
        for item in closest.entries() {
            todo.push_back(item);
        }

        Self {
            base_data: TaskData::new(dht),
            todo: Rc::new(RefCell::new(todo)),
            peer: Some(Box::new(value.clone())),
        }
    }
}

impl Task for ValueAnnounceTask {
    fn data(&self) -> &TaskData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut TaskData {
        &mut self.base_data
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self) {
        while self.can_request() {
            let cn = {
                match self.todo.borrow().front() {
                    Some(cn) => cn.clone(),
                    None => break,
                }
            };

            let msg = Rc::new(RefCell::new(store_value_req::Message::new()));
            let cloned_todo = self.todo.clone();
            if let Err(err) = self.send_call(cn, msg, Box::new(move|_| {
                cloned_todo.borrow_mut().pop_front();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }
}

impl fmt::Display for ValueAnnounceTask {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
