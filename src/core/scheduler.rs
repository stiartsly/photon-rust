use std::collections::LinkedList;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;
use tokio::time::{Duration, Instant};

struct Job {
    cb: Box<dyn FnMut()>,
    duration: Duration,
    periodic: bool,
}

impl Job {
    fn new<F>(cb: F, delay: u64 /* ms */, periodic: bool) -> Self
    where F: FnMut() + 'static {
        Self {
            cb: Box::new(cb),
            duration: Duration::from_millis(delay),
            periodic,
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

    timers: BTreeMap<Instant, LinkedList<Box<Job>>>,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Scheduler {
            updated: false,
            now: Instant::now(),
            timers: BTreeMap::new(),
        }
    }

    pub(crate) fn add_one_time<F>(&mut self, cb: F, start: u64, delay: u64)
    where F: FnMut() + 'static {
        self.add_job(
            Duration::from_millis(start),
            Box::new(Job::new(cb, delay, false ))
        );
    }

    pub(crate) fn add<F>(&mut self, cb: F, start: u64, delay: u64)
    where F: FnMut() + 'static {
        self.add_job(
            Duration::from_millis(start),
            Box::new(Job::new(cb, delay, true ))
        );
    }

    fn add_job(&mut self, start: Duration, job: Box<Job>) {
        let start = self.now + start;

        match self.timers.get_mut(&start) {
            Some(timer) => timer.push_back(job),
            None => {
                let mut timer = LinkedList::new();
                timer.push_back(job);
                self.timers.insert(start, timer);
            }
        }
        self.updated = true;
    }

    fn pop_jobs(&mut self) -> Option<LinkedList<Box<Job>>> {
        self.timers.pop_first().map(|(_,v)| v)
    }

    fn sync_time(&mut self) {
        self.now = Instant::now();
    }

    pub(crate) fn is_updated(&self) -> bool {
        self.updated
    }

    pub(crate) fn next_timeout(&self) -> Instant {
        match self.timers.iter().next() {
            Some(timer) => timer.0.clone(),
            None => {
                self.now + Duration::from_secs(60*60)
            }
        }
    }
}

pub(crate) fn run_jobs(sched: Rc<RefCell<Scheduler>>) {
    sched.borrow_mut().sync_time();

    let mut timer = match sched.borrow_mut().pop_jobs() {
        Some(v) => v,
        None => return
    };

    while let Some(mut job) = timer.pop_front() {
        job.cb();
        if job.periodic {
            sched.borrow_mut().add_job(job.duration, job);
        }
    }
}
