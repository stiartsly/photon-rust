use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;

use crate::{
    unwrap, as_millis,
    constants,
    Id,
    NodeInfo,
    dht::DHT,
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
    txid: i32,
    target: Rc<NodeInfo>,

    req: Option<Rc<RefCell<dyn Msg>>>,
    rsp: Option<Rc<RefCell<dyn Msg>>>,

    sent: SystemTime,
    responsed: SystemTime,

    state: State,

    state_changed_fn: Box<dyn Fn(&RpcCall, &State, &State)>,
    responsed_fn: Box<dyn Fn(&RpcCall, Rc<RefCell<dyn Msg>>)>,
    stalled_fn: Box<dyn Fn(&RpcCall)>,
    timeout_fn: Box<dyn Fn(&RpcCall)>,

    dht: Rc<RefCell<DHT>>,
}

static mut NEXT_TXID: i32= 0;

fn next_txid() -> i32 {
    unsafe {
        NEXT_TXID += 1;
        if NEXT_TXID == 0 {
            NEXT_TXID += 1;
        }
        NEXT_TXID
    }
}

#[allow(dead_code)]
impl RpcCall {
    pub(crate) fn new(
        dht: Rc<RefCell<DHT>>,
        target: &Rc<NodeInfo>,
        msg: Rc<RefCell<dyn Msg>>) -> Self
    {

        msg.borrow_mut().set_remote(
            target.id(),
            target.socket_addr()
        );

        RpcCall {
            txid: next_txid(),
            target: target.clone(),
            req: Some(msg),
            rsp: None,

            sent: SystemTime::UNIX_EPOCH,
            responsed: SystemTime::UNIX_EPOCH,
            state: State::Unsent,

            state_changed_fn: Box::new(|_, _, _| {}),
            responsed_fn: Box::new(|_, _| {}),
            stalled_fn: Box::new(|_| {}),
            timeout_fn: Box::new(|_| {}),

            dht,
        }
    }

    pub(crate) fn txid(&self) -> i32 {
        self.txid
    }

    pub(crate) fn dht(&self) -> Rc<RefCell<DHT>> {
        self.dht.clone()
    }

    pub(crate) fn target_id(&self) -> &Id {
        self.target.id()
    }

    pub(crate) fn target(&self) -> Rc<NodeInfo> {
        self.target.clone()
    }

    pub(crate) fn matches_id(&self) -> bool {
        self.rsp.as_ref().and_then(|rsp| {
            Some(rsp.borrow().id() == self.target_id())
        }).unwrap_or(false)
    }

    pub(crate) fn matches_addr(&self) -> bool {
        self.req.as_ref().and_then(|req| {
            self.rsp.as_ref().map(|rsp| {
                rsp.borrow().origin() == req.borrow().remote_addr()
            })
        }).unwrap_or(false)
    }

    pub(crate) fn req(&self) ->Option<Rc<RefCell<dyn Msg>>> {
        self.req.as_ref().cloned()
    }

    pub(crate) fn rsp(&self) -> Option<Rc<RefCell<dyn Msg>>>  {
        self.rsp.as_ref().cloned()
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
    where F: Fn(&RpcCall, Rc<RefCell<dyn Msg>>) + 'static {
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
                if let Some(rsp) = self.rsp() {
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

    pub(crate) fn responsed(&mut self, rsp: Rc<RefCell<dyn Msg>>) {
        self.rsp = Some(rsp);
        self.responsed = SystemTime::now();

        match unwrap!(self.rsp()).borrow().kind() {
            msg::Kind::Request => {},
            msg::Kind::Response => self.update_state(State::Responsed),
            msg::Kind::Error => self.update_state(State::Err)
        }
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
