use std::rc::Rc;
use std::boxed::Box;
use std::time::SystemTime;

use crate::node::Node;
use crate::rpcserver::RpcServer;
use crate::msg::msg::{self, Msg};
use crate::id::Id;

use log::{warn};

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum State {
    UNSENT,
    SENT,
    STALLED,
    TIMEOUT,
    CANCELED,
    ERR,
    RESPONDED
}

#[allow(dead_code)]
pub(crate) struct RpcCall {
    target: Node,

    req: Box<dyn Msg>,
    rsp: Option<Box<dyn Msg>>,

    sent_time: SystemTime,
    resped_time: SystemTime,

    state: State,

    on_state_change_fn: Box<dyn Fn(&Self, &State, &State)>,
    on_response_fn: Box<dyn Fn(&Self, &Box<dyn Msg>)>,
    on_stall_fn: Box<dyn Fn(&Self)>,
    on_timeout_fn: Box<dyn Fn(&Self)>
}

#[allow(dead_code)]
impl RpcCall {
    pub(crate) fn new(node: &Node, req: Box<dyn Msg>) -> Self {
        RpcCall {
            target: node.clone(),
            req, rsp: None,

            sent_time: SystemTime::UNIX_EPOCH,
            resped_time: SystemTime::UNIX_EPOCH,
            state: State::UNSENT,

            on_state_change_fn: Box::new(|_, _,_| {}),
            on_response_fn: Box::new(|_,_| {}),
            on_stall_fn: Box::new(|_| {}),
            on_timeout_fn: Box::new(|_|{})
        }
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
        match self.rsp.as_ref() {
            Some(msg) => msg.addr() == self.req.addr(),
            None => false
        }
    }

    pub(crate) fn request(&self) -> &Box<dyn Msg> {
        &self.req
    }

    pub(crate) fn response(&self) -> &Option<Box<dyn Msg>> {
        &self.rsp
    }

    pub(crate) fn sent_time(&self) -> &SystemTime {
        &self.sent_time
    }

    pub(crate) fn responsed_time(&self) -> &SystemTime {
        &self.resped_time
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn is_pending(&self) -> bool {
        unimplemented!()
    }

    pub(crate) fn set_state_change_fn<F>(&mut self, f: F)
    where F: Fn(&Self, &State, &State) + 'static {
        self.on_state_change_fn = Box::new(f)
    }

    pub(crate) fn set_response_fn<F>(&mut self, f: F)
    where F: Fn(&Self, &Box<dyn Msg>) + 'static {
        self.on_response_fn = Box::new(f)
    }

    pub(crate) fn set_stall_fn<F>(&mut self, f: F)
    where F: Fn(&Self) + 'static {
        self.on_stall_fn = Box::new(f)
    }

    pub(crate) fn set_timeout_fn<F>(&mut self, f: F)
    where F: Fn(&Self) + 'static {
        self.on_timeout_fn = Box::new(f)
    }

    /*
    void RPCCall::updateState(State currentState) {
    auto prevState {this->state};
    this->state = currentState;

    stateChangeHandler(this, prevState, currentState);

    switch (currentState) {
    case State::TIMEOUT:
        timeoutHandler(this);
        break;
    case State::STALLED:
        stallHandler(this);
        break;
    case State::RESPONDED:
        responseHandler(this, response);
        break;
    default:
        break;
    }
}
*/

    pub(crate) fn update_state(&mut self, new: State) {
        let old = self.state.clone();
        self.state = new;

        (self.on_state_change_fn)(self, &old, &self.state);

        match self.state {
            State::TIMEOUT => (self.on_timeout_fn)(self),
            State::STALLED => (self.on_stall_fn)(self),
            State::RESPONDED => {
                (self.on_response_fn)(self, self.rsp.as_ref().unwrap())
            }
            _ => {}
        }
    }

    pub(crate) fn sent(&self, _: &Rc<RpcServer>) {
        unimplemented!()
    }

    pub(crate) fn responsed(&mut self, response: Box<dyn Msg>) {
        assert!(response.kind() == msg::Kind::Response ||
                response.kind() == msg::Kind::Error);

        /*
        TODO check timer.
        */

        self.rsp = Some(response);
        self.resped_time = SystemTime::now();

        match self.rsp.as_ref().unwrap().kind() {
            msg::Kind::Response => self.update_state(State::RESPONDED),
            msg::Kind::Error => self.update_state(State::ERR),
            _ => {
                warn!("Unexpected message type received");
            }
        }
    }

    fn failed(&mut self) {
        self.update_state(State::TIMEOUT)
    }

    fn cancel(&mut self) {
        // TOOD: timeout Timer.

        self.update_state(State::CANCELED);
    }

    fn stall(&mut self) {
        if self.state == State::SENT {
            self.update_state(State::STALLED)
        }
    }

    fn check_timeout(&self) {
        unimplemented!()
    }
}
