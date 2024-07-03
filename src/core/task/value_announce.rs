use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;
use log::error;

use super::{
    closest_set::ClosestSet,
    candidate_node::CandidateNode,
    task::{Task, TaskData},
};

use crate::msg::{
    store_value_req
};

use crate::{
    value::Value,
};

#[allow(dead_code)]
pub(crate) struct ValueAnnounceTask {
    base_data: TaskData,

    todo: Rc<RefCell<LinkedList<Rc<RefCell<CandidateNode>>>>>,
    peer: Option<Box<Value>>,
}

#[allow(dead_code)]
impl ValueAnnounceTask {
    pub(crate) fn new(closest: &ClosestSet, value: &Value) -> Self {
        let mut todo = LinkedList::new();
        for item in closest.entries() {
            todo.push_back(item);
        }

        Self {
            base_data: TaskData::new(),
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
                let todo = self.todo.borrow();
                let cn = match todo.front() {
                    Some(cn) => cn,
                    None => break,
                };
                Rc::clone(&cn)
            };

            let req = Rc::new(RefCell::new(store_value_req::Message::new()));

            let todo = Rc::clone(&self.todo);
            if let Err(err) = self.send_call(cn, req, Box::new(move|_| {
                todo.borrow_mut().pop_front();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }
}
