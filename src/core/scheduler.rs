use std::collections::LinkedList;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;
use tokio::time::{Duration, Instant};

struct Job {
    cb: Box<dyn FnMut()>,
    period_time: Option<Duration>,
}

impl Job {
    fn new<F>(cb: F, period: u64 /* ms */) -> Self
    where F: FnMut() + 'static {
        let mut period_time = None;
        if period > 0 {
            period_time = Some(Duration::from_millis(period));
        }

        Self {
            cb: Box::new(cb),
            period_time,
        }
    }

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

    pub(crate) fn add_oneshot<F>(&mut self, cb: F, start: u64)
    where F: FnMut() + 'static {
        self.add_job(
            Duration::from_millis(start),
            Box::new(Job::new(cb, 0))
        );
    }

    pub(crate) fn add<F>(&mut self, cb: F, start: u64, period: u64)
    where F: FnMut() + 'static {
        self.add_job(
            Duration::from_millis(start),
            Box::new(Job::new(cb, period)),
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
            None => self.now + Duration::from_secs(60*60)
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
        let time = match job.period_time {
            Some(v) => v,
            None => continue,
        };
        sched.borrow_mut().add_job(time, job);
    }
}
