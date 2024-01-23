use std::fmt;
use std::time::SystemTime;

use crate::rpccall::RpcCall;
use crate::msg::msg::Msg;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    INITIAL,
    QUEUED,
    RUNNING,
    FINISHED,
    CANCELED
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            State::INITIAL => "INITIAL",
            State::QUEUED => "QUEUED",
            State::RUNNING => "RUNNING",
            State::FINISHED => "FINISHED",
            State::CANCELED => "CANCELED"
        };
        write!(f, "{}", str)?;
        Ok(())
    }
}

pub(crate) trait Task {
    fn taskid(&self) -> i32;
    fn name(&self) -> &str;
    fn state(&self) -> State;

    fn nested(&self) -> &Box<dyn Task>;

    fn is_canceled(&self) -> bool;
    fn is_finished(&self) -> bool;

    fn started_time(&self) -> &SystemTime;
    fn finished_time(&self) -> &SystemTime;

    fn age(&self) -> u128;

    fn set_nested(&mut self, _: Box<dyn Task>);

    fn start(&mut self);
    fn cancel(&mut self);

    fn call_sent(&mut self, _: &Box<RpcCall>);
    fn call_responsed(&mut self, _: &Box<RpcCall>, _: &Box<dyn Msg>);
    fn call_error(&mut self, _: &Box<RpcCall>);
    fn call_timeout(&mut self, _: &Box<RpcCall>);

    fn prepare(&mut self);
    fn update(&mut self);

    fn is_done(&self) -> bool;
}
