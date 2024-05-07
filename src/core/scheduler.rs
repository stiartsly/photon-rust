use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;

use tokio::time::{Duration, Instant};

struct Job {
    cb: Box<dyn FnMut()>,
    duration: Duration,
}

impl Job {
    fn new<F>(f: F, delay: u64 /* ms */) -> Self
    where F: FnMut() + 'static {
        Self {
            cb: Box::new(f),
            duration: Duration::from_millis(delay),
        }
    }

    /*fn cancel(&mut self) {
        self.cb = Box::new(||{});
    }*/

    pub(crate) fn cb(&mut self) {
        (self.cb)()
    }
}

pub(crate) struct Scheduler {
    updated: bool,
    now: Instant,
    timers: BTreeMap<Instant, Vec<Box<Job>>>,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Scheduler {
            updated: false,
            now: Instant::now(),
            timers: BTreeMap::new(),
        }
    }

    pub(crate) fn add<F>(&mut self, cb: F, start: u64, delay: u64)
    where F: FnMut() + 'static {
        self.add_job(
            Duration::from_millis(start),
            Box::new(Job::new(cb, delay))
        );
    }

    fn add_job(&mut self, start: Duration, job: Box<Job>) {
        let time = self.now + start;

        match self.timers.get_mut(&time) {
            Some(timer) => {
                timer.push(job);
            },
            None => {
                let mut timer = Vec::new();
                timer.push(job);
                _ = self.timers.insert(time, timer);
            }
        }
        self.updated = true;
    }

    fn pop_jobs(&mut self) -> Option<Vec<Box<Job>>> {
        self.timers.pop_first().map(|(_,v)| v)
    }

    fn sync_time(&mut self) {
        self.now = Instant::now();
    }

    pub(crate) fn is_updated(&self) -> bool {
        self.updated
    }

    pub(crate) fn next_time(&self) -> Instant {
        match self.timers.iter().next() {
            Some(timer) => {
                timer.0.clone()
            },
            None => {
                self.now + Duration::from_secs(60*60)
            }
        }
    }
}

pub(crate) fn run_jobs(scheduler: &Rc<RefCell<Scheduler>>) {
    let scheduler = Rc::clone(scheduler);
    let jobs = {
        scheduler.borrow_mut().sync_time();
        scheduler.borrow_mut().pop_jobs()
    };

    if let Some(mut jobs) = jobs {
        while let Some(mut job) = jobs.pop() {
            job.cb();
            scheduler.borrow_mut().add_job(job.duration.clone(),job);
        }
    }
}
