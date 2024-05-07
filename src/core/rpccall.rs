use std::fmt;
use std::time::SystemTime;

use crate::{
    unwrap, as_millis,
    constants,
    id::Id,
    node_info::NodeInfo,
    msg::msg::{self, Msg}
};

#[derive(Clone, PartialEq, PartialOrd, Debug)]
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
    hashid: i32,
    target: Box<NodeInfo>,

    req: Option<Box<dyn Msg>>,
    rsp: Option<Box<dyn Msg>>,

    sent: SystemTime,
    responsed: SystemTime,

    state: State,

    state_changed_fn: Box<dyn Fn(&RpcCall, &State, &State)>,
    responsed_fn: Box<dyn Fn(&RpcCall, &Box<dyn Msg>)>,
    stalled_fn: Box<dyn Fn(&RpcCall)>,
    timeout_fn: Box<dyn Fn(&RpcCall)>
}

static mut HASH_ID: i32 = 0;

#[allow(dead_code)]
impl RpcCall {
    pub(crate) fn new(ni: Box<NodeInfo>, req: Box<dyn Msg>) -> Self {
        let hash = unsafe {
            HASH_ID += 1;
            if HASH_ID >= i32::MAX {
                HASH_ID += 1;
            }
            HASH_ID
        };

        RpcCall {
            hashid: hash,
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

    pub(crate) fn hash(&self) -> i32 {
        self.hashid
    }

    pub(crate) fn target_id(&self) -> &Id {
        self.target.id()
    }

    pub(crate) fn target(&self) -> &Box<NodeInfo> {
        &self.target
    }

    pub(crate) fn matches_id(&self) -> bool {
        match self.rsp.as_ref() {
            Some(msg) => msg.id() == self.target.id(),
            None => false
        }
    }

    pub(crate) fn matches_addr(&self) -> bool {
        /*match self.rsp.as_ref() {
            Some(msg) => msg.addr() == self.target.socket_addr(),
            None => false
        }*/
        true
    }

    pub(crate) fn req(&mut self) ->Option<Box<dyn Msg>> {
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
    where F: Fn(&RpcCall, &State, &State) + 'static {
        self.state_changed_fn = Box::new(f)
    }

    pub(crate) fn set_responsed_fn<F>(&mut self, f: F)
    where F: Fn(&RpcCall, &Box<dyn Msg>) + 'static {
        self.responsed_fn = Box::new(f)
    }

    pub(crate) fn set_stalled_fn<F>(&mut self, f: F)
    where F: Fn(&RpcCall) + 'static {
        self.stalled_fn = Box::new(f)
    }

    pub(crate) fn set_timeout_fn<F>(&mut self, f: F)
    where F: Fn(&RpcCall) + 'static {
        self.timeout_fn = Box::new(f)
    }

    pub(crate) fn update_state(&mut self, new_state: State) {
        let prev_state = self.state.clone();
        self.state = new_state;

        (self.state_changed_fn)(self, &prev_state, &self.state);
        match self.state {
            State::Timeout => (self.timeout_fn)(self),
            State::Stalled => (self.stalled_fn)(self),
            State::Responsed => {
                if let Some(rsp) = self.rsp.as_ref() {
                    (self.responsed_fn)(self, rsp)
                }
            }
            _ => {}
        }
    }

    pub(crate) fn send(&mut self) {
        self.sent = SystemTime::now();
        self.update_state(State::Sent);
    }

    pub(crate) fn responsed(&mut self, rsp: Box<dyn Msg>) -> Box<dyn Msg> {
        self.rsp = Some(rsp);
        self.responsed = SystemTime::now();

        match unwrap!(self.rsp).kind() {
            msg::Kind::Request => {},
            msg::Kind::Response => self.update_state(State::Responsed),
            msg::Kind::Error => self.update_state(State::Err)
        }

        self.rsp.take().unwrap()
    }

    fn failed(&mut self) {
        self.update_state(State::Timeout)
    }

    fn cancel(&mut self) {
        // TOOD: cancel checking timeout.
        self.update_state(State::Canceled);
    }

    pub(crate) fn stall(&mut self) {
        if self.state != State::Sent {
            self.update_state(State::Stalled)
        }
    }

    pub(crate) fn check_timeout(&mut self) {
        if self.state != State::Sent && self.state != State::Stalled {
            return;
        }

        if constants::RPC_CALL_TIMEOUT_MAX > as_millis!(&self.sent) {
            self.update_state(State::Stalled);
            // TODO:
        } else {
            self.update_state(State::Timeout);
        }
    }
}

impl fmt::Display for RpcCall {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
