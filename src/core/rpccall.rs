use std::rc::Rc;
use std::cell::RefCell;
use std::boxed::Box;
use std::time::SystemTime;
use log::warn;

use crate::{
    id::Id,
    dht::DHT,
    node::Node,
    msg::msg,
    msg::msg::Msg,
    rpcserver::RpcServer
};

#[derive(Clone, PartialEq, PartialOrd, Hash)]
pub(crate) enum State {
    Unsent,
    Sent,
    Stalled,
    Timeout,
    Canceled,
    Err,
    Responsed
}

pub(crate) struct RpcCall {
    //dht: Option<Rc<RefCell<DHT>>>,
    target: Node,

    req: Box<dyn Msg>,
    rsp: Option<Box<dyn Msg>>,

    sent: SystemTime,
    responsed: SystemTime,

    state: State,

    state_changed_fn: Box<dyn Fn(&Self, &State, &State)>,
    responsed_fn: Box<dyn Fn(&Self, &Box<dyn Msg>)>,
    stalled_fn: Box<dyn Fn(&Self)>,
    timeout_fn: Box<dyn Fn(&Self)>
}

#[allow(dead_code)]
impl RpcCall {
    pub(crate) fn new(node: &Node, req: Box<dyn Msg>) -> Self {
        RpcCall {
            //dht: None,
            target: node.clone(),
            req, rsp: None,

            sent: SystemTime::UNIX_EPOCH,
            responsed: SystemTime::UNIX_EPOCH,
            state: State::Unsent,

            state_changed_fn: Box::new(|_, _,_| {}),
            responsed_fn: Box::new(|_,_| {}),
            stalled_fn: Box::new(|_| {}),
            timeout_fn: Box::new(|_|{})
        }
    }

    pub(crate) fn dht(&self) -> &Rc<RefCell<DHT>> {
        unimplemented!()
    }

    pub(crate) fn target_id(&self) -> &Id {
        self.target.id()
    }

    pub(crate) fn target(&self) -> &Node {
        &self.target
    }

    pub(crate) fn matches_id(&self) -> bool {
        self.req.id() == self.target_id()
    }

    pub(crate) fn matches_address(&self) -> bool {
        if let Some(msg) = self.rsp.as_ref() {
            msg.addr() == self.req.addr()
        } else {
            false
        }
    }

    pub(crate) fn req(&self) -> &Box<dyn Msg> {
        &self.req
    }

    pub(crate) fn rsp(&self) -> &Option<Box<dyn Msg>> {
        &self.rsp
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
    where F: Fn(&Self, &State, &State) + 'static {
        self.state_changed_fn = Box::new(f)
    }

    pub(crate) fn set_responsed_fn<F>(&mut self, f: F)
    where F: Fn(&Self, &Box<dyn Msg>) + 'static {
        self.responsed_fn = Box::new(f)
    }

    pub(crate) fn set_stalled_fn<F>(&mut self, f: F)
    where F: Fn(&Self) + 'static {
        self.stalled_fn = Box::new(f)
    }

    pub(crate) fn set_timeout_fn<F>(&mut self, f: F)
    where F: Fn(&Self) + 'static {
        self.timeout_fn = Box::new(f)
    }

    pub(crate) fn update_state(&mut self, new: State) {
        let old = self.state.clone();
        self.state = new;

        (self.state_changed_fn)(self, &old, &self.state);

        match self.state {
            State::Timeout => (self.timeout_fn)(self),
            State::Stalled => (self.stalled_fn)(self),
            State::Responsed => (self.responsed_fn)(self, self.rsp.as_ref().unwrap()),
            _ => {}
        }
    }

    pub(crate) fn send(&mut self, _: &RpcServer) {
        self.sent = SystemTime::now();
        self.update_state(State::Sent);

        // Timer
    }

    pub(crate) fn responsed(&mut self, response: Box<dyn Msg>) {
        assert!(response.kind() == msg::Kind::Response ||
                response.kind() == msg::Kind::Error);

        /*
        TODO check timer.
        */

        self.rsp = Some(response);
        self.responsed = SystemTime::now();

        match self.rsp.as_ref().unwrap().kind() {
            msg::Kind::Response => self.update_state(State::Responsed),
            msg::Kind::Error => self.update_state(State::Err),
            _ => {
                warn!("Unexpected message type received");
            }
        }
    }

    fn failed(&mut self) {
        self.update_state(State::Timeout)
    }

    fn cancel(&mut self) {
        // TOOD: timeout Timer.

        self.update_state(State::Canceled);
    }

    fn stall(&mut self) {
        if self.state == State::Sent {
            self.update_state(State::Stalled)
        }
    }

    fn check_timeout(&self) {
        unimplemented!()
    }
}
