use std::boxed::Box;
use crate::msg::message::Message;
use crate::id::Id;

#[allow(dead_code)]
enum State {
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
    request: Box<dyn Message>,
    response: Box<dyn Message>,

    on_state_change: Box<dyn Fn(&Self, &State)>,
    on_response: Box<dyn Fn(&Self, Box<dyn Message>)>,
    on_stall: Box<dyn Fn(&Self)>,
    on_timeout: Box<dyn Fn(&Self)>
}

#[allow(dead_code)]
impl RpcCall {
    pub(crate) fn new(req: Box<dyn Message>, rsp: Box<dyn Message>) -> Self {
        RpcCall {
            request: req,
            response: rsp,
            on_state_change: Box::new(|_, _| {}),
            on_response: Box::new(|_,_| {}),
            on_stall: Box::new(|_| {}),
            on_timeout: Box::new(|_|{})
        }
    }

    pub(crate) fn request(&self) -> &Box<dyn Message> {
        &self.request
    }

    pub(crate) fn response(&self) -> &Box<dyn Message> {
        &self.response
    }

    pub(crate) fn id(&self) -> &Id {
        unimplemented!()
    }
}
