use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;

use super::{
    closest_set::ClosestSet,
    candidate_node::CandidateNode,
    task::{Task, TaskData},
};
use crate::{
    value::Value,
};

#[allow(dead_code)]
pub(crate) struct ValueAnnounceTask {
    base_data: TaskData,

    // TODO: todo: Rc<RefCell<LinkedList<Rc<RefCell<KBucketEntry>>>>>,
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
        /*
        while !self.todo.borrow().is_empty() && self.can_request() {
            let candidate_node = match self.todo.borrow().front() {
                Some(node) => node,
                None => break,
            };

            let req = Rc::new(RefCell::new(announce_value_req::Message::new()));

            let cloned = Rc::clone(candidate_node);
            let cloned_todo = Rc::clone(&self.todo);
            if let Err(err) = self.send_call(cloned, req, Box::new(move|_| {
                cloned_todo.borrow_mut().pop_front();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }*/
        unimplemented!()
    }
}
