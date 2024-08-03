use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;
use std::collections::HashMap;
use log::debug;

use crate::{
    unwrap,
    NodeInfo,
    rpccall::{RpcCall, State as CallState},
    dht::DHT,
    error::Error,
    msg::msg::Msg,
    routing_table::RoutingTable,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Initial,
    Queued,
    Running,
    Finished,
    Canceled,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            State::Initial => "INITIAL",
            State::Queued => "QUEUED",
            State::Running => "RUNNING",
            State::Finished => "FINISHED",
            State::Canceled => "CANCELED",
        };
        write!(f, "{}", str)?;
        Ok(())
    }
}

pub(crate) type TaskId = i32;
static mut NEXT_TASKID: TaskId= 0;

fn next_taskid() -> TaskId {
    unsafe {
        NEXT_TASKID += 1;
        if NEXT_TASKID == 0 {
            NEXT_TASKID += 1;
        }
        NEXT_TASKID
    }
}

pub(crate) struct TaskData {
    taskid: TaskId,
    name: String,
    state: State,

    started_time:  SystemTime,
    finished_time: SystemTime,

    inflights: HashMap<TaskId, Rc<RefCell<RpcCall>>>,
    listeners: Vec<Box<dyn FnOnce(Rc<RefCell<dyn Task>>)>>,

    nested: Option<Rc<RefCell<dyn Task>>>,
    cloned: Option<Rc<RefCell<dyn Task>>>,

    dht: Rc<RefCell<DHT>>,
}

impl TaskData {
    pub(crate) fn new(dht: Rc<RefCell<DHT>>) -> Self {
        Self {
            taskid: next_taskid(),
            name: "".to_string(),
            state: State::Initial,

            started_time: SystemTime::UNIX_EPOCH,
            finished_time: SystemTime::UNIX_EPOCH,

            inflights: HashMap::new(),
            listeners: Vec::new(),

            nested: None,
            cloned: None,
            dht,
        }
    }

    fn notify_completion(&mut self, task: Rc<RefCell<dyn Task>>) {
        while let Some(f) = self.listeners.pop() {
            f(task.clone())
        }
    }

    pub(crate) fn rt(&self) -> Rc<RefCell<RoutingTable>> {
        self.dht.borrow().routing_table()
    }

    pub(crate) fn dht(&self) -> Rc<RefCell<DHT>> {
        self.dht.clone()
    }

    pub(crate) fn task(&self) -> Rc<RefCell<dyn Task>> {
        unwrap!(self.cloned).clone()
    }
}

pub(crate) trait Task {
    fn data(&self) -> &TaskData;
    fn data_mut(&mut self) -> &mut TaskData;

    fn prepare(&mut self) {}
    fn update(&mut self) {}
    fn call_sent(&mut self, _: &RpcCall) {}
    fn call_responsed(&mut self, _: &RpcCall, _: Rc<RefCell<dyn Msg>>) {}
    fn call_error(&mut self, _: &RpcCall) {}
    fn call_timeout(&mut self, _: &RpcCall) {}
    fn as_any(&self) -> &dyn Any;

    fn set_cloned(&mut self, task: Rc<RefCell<dyn Task>>) {
        self.data_mut().cloned = Some(task)
    }

    fn taskid(&self) -> i32 {
        self.data().taskid
    }

    fn name(&self) -> &str {
        &self.data().name
    }

    fn set_name(&mut self, name: &str) {
        self.data_mut().name = name.to_string()
    }

    fn state(&self) -> State {
        self.data().state
    }

    fn set_state(&mut self, expected:&[State], new_state: State) -> bool {
        let found = expected.contains(&self.state());
        if found {
            self.data_mut().state = new_state;
        }
        found
    }

    fn nested(&self) -> Option<Rc<RefCell<dyn Task>>> {
        self.data().nested.as_ref().map(|v| v.clone())
    }

    fn set_nested(&mut self, nested: Rc<RefCell<dyn Task>>) {
        self.data_mut().nested = Some(nested);
    }

    fn add_listener(&mut self, _: Box<dyn FnOnce(Rc<RefCell<dyn Task>>)>) {
        //TODO: unimplemented!()
    }

    fn start(&mut self) {
        if self.set_state(&[State::Queued], State::Running) {
            self.data_mut().started_time = SystemTime::now();
            self.prepare();
            self.update();
        }

        if self.is_done() &&
            self.set_state(&[State::Running], State::Finished) {
            let cloned_task = self.data().task();

            self.data_mut().finished_time = SystemTime::now();
            self.data_mut().notify_completion(cloned_task);
        }
    }

    fn cancel(&mut self) {
        let expected = vec![
            State::Initial,
            State::Queued,
            State::Running
        ];

        if self.set_state(&expected, State::Canceled) {
            let cloned_task = self.data().task();

            self.data_mut().finished_time = SystemTime::now();
            self.data_mut().notify_completion(cloned_task);
        }

        if let Some(nested) = self.data_mut().nested.as_mut() {
            nested.borrow_mut().cancel()
        }
    }

    fn is_done(&self) -> bool {
        self.data().inflights.is_empty()
    }

    fn is_canceled(&self) -> bool {
        self.data().state == State::Canceled
    }

    fn is_finished(&self) -> bool {
        self.data().state == State::Finished ||
            self.data().state == State::Canceled
    }

    fn can_request(&self) -> bool {
        self.data().inflights.len() < 10 && !self.is_finished()
    }

    fn send_call(&mut self,
        node: Rc<NodeInfo>,
        msg: Rc<RefCell<dyn Msg>>,
        mut cb: Box<dyn FnMut(Rc<RefCell<RpcCall>>)>)
    -> Result<(), Error> {
        if !self.can_request() {
            return Ok(())
        }

        let call = Rc::new(RefCell::new(RpcCall::new(node, self.data().dht(), msg)));
        let task = self.data().task();

        call.borrow_mut().set_cloned(call.clone());
        call.borrow_mut().set_state_changed_fn (move|c, _, cur| {
            let mut task = task.borrow_mut();
            let mut update_needed = true;
            match cur {
                CallState::Sent => task.call_sent(c),
                CallState::Responsed => {
                    update_needed = true;
                    task.data_mut().inflights.remove(&c.txid());
                    if !task.is_finished() && c.rsp().is_some() {
                        task.call_responsed(c, unwrap!(c.rsp()).clone());
                    }
                },
                CallState::Err => {
                    update_needed = true;
                    task.data_mut().inflights.remove(&c.txid());
                    if !task.is_finished() {
                        task.call_error(c);
                    }

                },
                CallState::Timeout => {
                    update_needed = true;
                    task.data_mut().inflights.remove(&c.txid());
                    if !task.is_finished() {
                        task.call_timeout(c);
                    }
                }
                CallState::Stalled => {
                    update_needed = true;
                }
                _ => {}
            }

            if update_needed {
                // task.update();
            }
        });

        (cb)(call.clone());
        self.data_mut().inflights.insert(call.borrow().txid(), call.clone());

        debug!("Task#{} sending call to {}{}",
            self.taskid(),
            self.data().dht.borrow().node_id(),
            self.data().dht.borrow().socket_addr()
        );

        let server = self.data().dht.borrow().server();
        server.borrow_mut().send_call(call);
        Ok(())
    }
}

impl fmt::Display for dyn Task {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
