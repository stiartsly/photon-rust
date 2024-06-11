use std::fmt;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;
use std::collections::HashMap;
//use log::debug;

use crate::{
    //node_info::NodeInfo,
    rpccall::{RpcCall, State as CallState},
    error::Error,
    msg::msg::Msg,
    server::Server,
};

use super::{
    candidate_node::CandidateNode,
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

    started_time: SystemTime,
    finished_time: SystemTime,

    inflights: HashMap<TaskId, Rc<RefCell<RpcCall>>>,
    listeners: Vec<Box<dyn FnOnce(&dyn Task)>>,

    nested: Option<Box<dyn Task>>,

    ref_task: Option<Rc<RefCell<dyn Task>>>,
    server: Option<Rc<RefCell<Server>>>,
}

#[allow(dead_code)]
impl TaskData {
    pub(crate) fn new() -> Self {
        Self {
            taskid: next_taskid(),
            name: "N/A".to_string(),
            state: State::Initial,
            nested: None,
            started_time: SystemTime::UNIX_EPOCH,
            finished_time: SystemTime::UNIX_EPOCH,
            inflights: HashMap::new(),
            listeners: Vec::new(),

            ref_task: None,
            server: None,
        }
    }

    fn notify_completion(&mut self, task: &dyn Task) {
        while let Some(f) = self.listeners.pop() {
            f(task)
        }
    }
}

pub(crate) trait Task {
    fn data(&self) -> &TaskData;
    fn data_mut(&mut self) -> &mut TaskData;

    fn prepare(&mut self);
    fn update(&mut self);
    fn call_sent(&mut self, _: &RpcCall);
    fn call_responsed(&mut self, call: &RpcCall, rsp: Rc<RefCell<dyn Msg>>);
    fn call_error(&mut self, call: &RpcCall);
    fn call_timeout(&mut self, call: &RpcCall);
    fn as_any(&self) -> &dyn Any;

    fn link_self(&mut self, task: Rc<RefCell<dyn Task>>) {
        self.data_mut().ref_task = Some(task)
    }

    fn link_server(&mut self, server: Rc<RefCell<Server>>)  {
        self.data_mut().server = Some(server)
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
        match found {
            true => {
                self.data_mut().state = new_state;
                true
            },
            false => false,
        }
    }

    fn nested(&self) -> Option<&Box<dyn Task>> {
        unimplemented!()
    }

    fn set_nested(&mut self, _: Box<dyn Task>) {
        unimplemented!()
    }

    fn add_listener(&mut self, _: Box<dyn FnOnce(&dyn Task)>) {
        //unimplemented!()
    }

    fn start(&mut self) {
        if self.set_state(&[State::Queued], State::Running) {
            self.data_mut().started_time = SystemTime::now();
            self.prepare();
            self.update();
        }

        if self.is_done() &&
            self.set_state(&[State::Running], State::Finished) {
            self.data_mut().finished_time = SystemTime::now();
            //self.data_mut().notify_completion();
        }
    }

    fn cancel(&mut self) {
        let expected = vec![
            State::Initial,
            State::Queued,
            State::Running
        ];
        if self.set_state(&expected, State::Canceled) {
            self.data_mut().finished_time = SystemTime::now();
            // self.data_mut().notify_completion(Box::new(self));
        }

        if let Some(nested) = self.data_mut().nested.as_mut() {
            nested.cancel()
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
        cn: Rc<RefCell<CandidateNode>>,
        msg: Rc<RefCell<dyn Msg>>,
        mut f: Box<dyn FnMut(Rc<RefCell<RpcCall>>)>)
    -> Result<(), Error> {
        if !self.can_request() {
            return Ok(())
        }

        let ni = Box::new(cn.borrow().node().clone());
        let call = Rc::new(RefCell::new(RpcCall::new(ni, msg)));
        let task = Rc::clone(self.data().ref_task.as_ref().unwrap());
        let server = Rc::clone(self.data().server.as_ref().unwrap());
        call.borrow_mut().set_state_changed_fn (move|call, prev_state, _| {
            match prev_state {
                CallState::Sent => task.borrow_mut().call_sent(call),
                CallState::Responsed => {
                    task.borrow_mut().data_mut().inflights.remove(&call.hash());
                    if task.borrow().is_done() {
                        if let Some(msg) = call.rsp() {
                            task.borrow_mut().call_responsed(call, msg);
                        }
                    }
                },
                CallState::Err => {
                    task.borrow_mut().data_mut().inflights.remove(&call.hash());
                    if task.borrow().is_done() {
                        task.borrow_mut().call_timeout(call);
                    }
                },
                CallState::Timeout => {}
                _ => {}
            }

            //if need_update {
            //    self.serialized_update()
            //}
            println!("state change invoked: prev: {:?} >>>>>>>>>>", prev_state);
        });

        (f)(Rc::clone(&call));
        self.data_mut().inflights.insert(call.borrow().hash(), Rc::clone(&call));

        // debug!("Task#{} sending call to {}{}", self.taskid(), node, msg.addr());
        server.borrow_mut().send_call(call);

        println!("send call>>>>>>");
        Ok(())
    }
}
