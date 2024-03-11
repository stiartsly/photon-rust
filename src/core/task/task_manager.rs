use std::collections::LinkedList;
use super::task::{
    Task,
    State
};

pub(crate) struct TaskManager {
    queued: LinkedList<Box<dyn Task>>,
    running: LinkedList<Box<dyn Task>>,

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

    pub(crate) fn add(&mut self, task: Box<dyn Task>) {
        self.add_prior(task, false)
    }

    pub(crate) fn add_prior(&mut self, mut task: Box<dyn Task>, prior: bool) {
        if self.canceling {
            return;
        }
        if task.state() == State::Running {
            self.running.push_back(task);
            return;
        }

        let expected = vec![State::Initial];
        if !task.set_state(&expected, State::Queued) {
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

            if let Some(mut task) = self.queued.pop_front() {
                if task.is_finished() {
                    continue;
                }

                task.start();
                if !task.is_finished() {
                    self.running.push_back(task);
                }
            }
        }
    }

    pub(crate) fn cancel_all(&mut self) {
        self.canceling = true;
        while let Some(mut task) = self.running.pop_front() {
            task.cancel();
        }
        while let Some(mut task) = self.queued.pop_front() {
            task.cancel();
        }
        self.canceling = false;
    }
}
