use std::collections::BTreeMap;

use tokio::time::{Duration, Instant};

#[allow(dead_code)]
struct Job {
    cb: Box<dyn FnMut()>,
    duration: Duration,
}

#[allow(dead_code)]
impl Job {
    fn new<F>(f: F, delay: u64 /* ms */) -> Self where F: FnMut() + 'static {
        Job {
            cb: Box::new(f),
            duration: Duration::from_millis(delay),
        }
    }

    fn cancel(&mut self) {
        self.cb = Box::new(||{});
    }

    fn handle(&mut self) {
        (self.cb)()
    }
}

pub(crate) struct Scheduler {
    updated: bool,
    now: Instant,
    timers: BTreeMap<Instant, Vec<Box<Job>>>,
}

#[allow(dead_code)]
impl Scheduler {
    pub(crate) fn new() -> Self {
        Scheduler {
            updated: false,
            now: Instant::now(),
            timers: BTreeMap::new(),
        }
    }

    pub(crate) fn add<F>(&mut self, start: u64, delay: u64, cb: F) where F: FnMut() + 'static {
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

    pub(crate) fn run(&mut self) {
        let mut to_remove = Option::default() as Option<Instant>;
        if let Some((time, jobs)) = self.timers.iter_mut().next() {
            jobs.iter_mut().for_each(|job | {
                (job.cb)()
            });
            to_remove = Some(time.clone());
        }

        if let Some(item) = to_remove {
            if let Some(mut jobs) = self.timers.remove(&item) {
                while !jobs.is_empty() {
                    let job = jobs.pop().unwrap();
                    self.add_job(job.duration.clone(), job);
                }
            }
        }
    }

    pub(crate) fn sync_time(&mut self) {
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
