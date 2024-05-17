use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;

use super::task::{
    Task,
    State
};

pub(crate) struct TaskManager {
    queued: LinkedList<Rc<RefCell<dyn Task>>>,
    running: LinkedList<Rc<RefCell<dyn Task>>>,

    canceling: bool,
}

impl TaskManager {
    pub(crate) fn new() -> Self {
        Self {
            queued: LinkedList::new(),
            running: LinkedList::new(),
            canceling: false,
        }
    }

    pub(crate) fn add(&mut self, task: Rc<RefCell<dyn Task>>) {
        self.add_prior(task, false)
    }

    pub(crate) fn add_prior(&mut self, task: Rc<RefCell<dyn Task>>, prior: bool) {
        if self.canceling {
            return;
        }
        if task.borrow().state() == State::Running {
            self.running.push_back(task);
            return;
        }

        let expected = vec![State::Initial];
        if !task.borrow_mut().set_state(&expected, State::Queued) {
            return;
        }

        match prior {
            true => self.queued.push_front(task),
            false => self.queued.push_back(task),
        }
    }

    pub(crate) fn dequeue(&mut self) {
        loop {
            if self.canceling || self.running.len() >= 16 {
                break;
            }
            if self.queued.is_empty() {
                break;
            }

            if let Some(task) = self.queued.pop_front() {
                if task.borrow().is_finished() {
                    continue;
                }

                task.borrow_mut().start();
                if !task.borrow().is_finished() {
                    self.running.push_back(task);
                }
            }
        }
    }

    pub(crate) fn cancel_all(&mut self) {
        self.canceling = true;
        while let Some(task) = self.running.pop_front() {
            task.borrow_mut().cancel();
        }
        while let Some(task) = self.queued.pop_front() {
            task.borrow_mut().cancel();
        }
        self.canceling = false;
    }
}
