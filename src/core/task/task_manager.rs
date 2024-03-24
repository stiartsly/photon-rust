use std::boxed::Box;

use crate::task::task::Task;

pub(crate) struct TaskManager {
}

#[allow(dead_code)]
impl TaskManager {
    pub(crate) fn new() -> Self {
        TaskManager {}
    }

    pub(crate) fn add(&self, _: Box<dyn Task>) {
        unimplemented!()
    }

    pub(crate) fn cancel_all(&mut self) {
        // unimplemented!()
    }

    pub(crate) fn dequeue(&mut self) {
        // unimplemented!()
    }
}
