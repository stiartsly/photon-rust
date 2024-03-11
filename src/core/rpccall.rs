use std::fmt;
use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

use crate::{
    id::Id,
    node_info::NodeInfo,
    dht::DHT,
    scheduler::Scheduler,
    msg::msg::{self, Msg}
};

#[derive(Clone, PartialEq, PartialOrd, Debug, Hash)]
pub(crate) enum State {
    Unsent,
    Sent,
    Stalled,
    Timeout,
    Canceled,
    Err,
    Responsed,
}

pub(crate) struct RpcCall {
    target: Box<NodeInfo>,

    req: Option<Box<dyn Msg>>,
    rsp: Option<Box<dyn Msg>>,

    sent: SystemTime,
    responsed: SystemTime,

    state: State,

    state_changed_fn: Box<dyn Fn(&RpcCall, &State, &State)>,
    responsed_fn: Box<dyn Fn(&RpcCall, Option<&Box<dyn Msg>>)>,
    stalled_fn: Box<dyn Fn(&RpcCall)>,
    timeout_fn: Box<dyn Fn(&RpcCall)>
}

#[allow(dead_code)]
impl RpcCall {
    pub(crate) fn new(ni: Box<NodeInfo>, req: Box<dyn Msg>) -> Self {
        RpcCall {
            target: ni,
            req: Some(req),
            rsp: None,

            sent: SystemTime::UNIX_EPOCH,
            responsed: SystemTime::UNIX_EPOCH,
            state: State::Unsent,

            state_changed_fn: Box::new(|_, _, _| {}),
            responsed_fn: Box::new(|_, _| {}),
            stalled_fn: Box::new(|_| {}),
            timeout_fn: Box::new(|_| {}),
        }
    }

    pub(crate) fn dht(&self) -> &Rc<RefCell<DHT>> {
        unimplemented!()
    }

    pub(crate) fn target_id(&self) -> &Id {
        self.target.id()
    }

    pub(crate) fn target(&self) -> &NodeInfo {
        &self.target
    }

    pub(crate) fn matches_id(&self) -> bool {
        match self.rsp.as_ref() {
            Some(msg) => {
                println!(" id-1: {}", msg.id());
                println!(" id-2: {}", self.target_id());
                msg.id() == self.target_id()
            },
            None => false
        }
    }

    pub(crate) fn matches_address(&self) -> bool {
        /*match self.rsp.as_ref() {
            Some(msg) => msg.addr() == self.req.addr(),
            None => false
        }*/
        true
    }

    pub(crate) fn req_take(&mut self) -> Option<Box<dyn Msg>> {
        self.req.take()
    }

    pub(crate) fn rsp(&self) -> Option<&Box<dyn Msg>> {
        self.rsp.as_ref()
    }

    pub(crate) fn sent_time(&self) -> &SystemTime {
        &self.sent
    }

    pub(crate) fn responsed_time(&self) -> &SystemTime {
        &self.responsed
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn is_pending(&self) -> bool {
        self.state < State::Timeout
    }

    pub(crate) fn set_state_changed_fn<F>(&mut self, f: F)
    where F: Fn(&RpcCall, &State, &State) + 'static
    {
        self.state_changed_fn = Box::new(f)
    }

    pub(crate) fn set_responsed_fn<F>(&mut self, f: F)
    where F: Fn(&RpcCall, Option<&Box<dyn Msg>>) + 'static
    {
        self.responsed_fn = Box::new(f)
    }

    pub(crate) fn set_stalled_fn<F>(&mut self, f: F)
    where F: Fn(&RpcCall) + 'static,
    {
        self.stalled_fn = Box::new(f)
    }

    pub(crate) fn set_timeout_fn<F>(&mut self, f: F)
    where F: Fn(&RpcCall) + 'static,
    {
        self.timeout_fn = Box::new(f)
    }

    pub(crate) fn update_state(&mut self, new: State) {
        let old = self.state.clone();
        self.state = new;

        (self.state_changed_fn)(self, &old, &self.state);

        match self.state {
            State::Timeout => (self.timeout_fn)(self),
            State::Stalled => (self.stalled_fn)(self),
            State::Responsed => (self.responsed_fn)(self, self.rsp.as_ref()),
            _ => {}
        }
    }

    pub(crate) fn send(&mut self, _: &Rc<RefCell<Scheduler>>) {
        self.sent = SystemTime::now();
        self.update_state(State::Sent);

        // Timer
    }

    pub(crate) fn responsed(&mut self, response: Box<dyn Msg>) -> Box<dyn Msg> {
        /*
        TODO check timer.
        */

        self.rsp = Some(response);
        self.responsed = SystemTime::now();

        match self.rsp.as_ref().unwrap().kind() {
            msg::Kind::Request => {},
            msg::Kind::Response => self.update_state(State::Responsed),
            msg::Kind::Error => self.update_state(State::Err)
        }

        self.rsp.take().unwrap()
    }

    fn failed(&mut self) {
        self.update_state(State::Timeout)
    }

    pub(crate) fn response_socket_mismatch(&self) {}

    fn cancel(&mut self) {
        // TOOD: timeout Timer.

        self.update_state(State::Canceled);
    }

    pub(crate) fn stall(&mut self) {
        if self.state == State::Sent {
            self.update_state(State::Stalled)
        }
    }

    fn check_timeout(&self) {
        unimplemented!()
    }
}

impl fmt::Display for RpcCall {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
