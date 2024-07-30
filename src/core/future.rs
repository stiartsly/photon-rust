use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::future::Future;

use crate::{
    Id,
    LookupOption,
    Peer,
    Value,
    NodeInfo,
    Compound,
    error::Error,
};

pub(crate) struct FindNodeCmd {
    id: Id,
    option: LookupOption,
    result: Option<Result<Compound<NodeInfo>, Error>>,
    waker: Option<Waker>,
    completed: bool,
}

impl FindNodeCmd {
    pub(crate) fn new(id: &Id, option: &LookupOption) -> Self {
        Self {
            id: id.clone(),
            option: option.clone(),
            result: None,
            waker: None,
            completed: false,
        }
    }

    pub(crate) fn id(&self) -> &Id {
        &self.id
    }

    pub(crate) fn option(&self) -> LookupOption {
        self.option
    }

    pub(crate) fn result(&mut self) -> Result<Compound<NodeInfo>, Error> {
        self.result.take().unwrap()
    }

   // pub(crate) fn has_result(&self) -> bool {
   //     self.result.is_some()
   // }

    pub(crate) fn complete(&mut self, result: Result<Compound<NodeInfo>, Error>) {
        if let Some(waker) = self.waker.take() {
            self.result = Some(result);
            self.completed = true;
            waker.wake()
        }
    }

    fn is_completed(&self) -> bool {
        self.completed
    }

    fn set_waker(&mut self, waker: Waker) {
        self.completed = false;
        self.waker = Some(waker);
    }
}

#[allow(dead_code)]
pub(crate) struct FindValueCmd {
    id: Id,
    option: LookupOption,

    result: Option<Result<Option<Value>, Error>>,

    waker: Option<Waker>,
    completed: bool,
}

#[allow(dead_code)]
impl FindValueCmd {
    pub(crate) fn new(id: &Id, option: &LookupOption) -> Self {
        Self {
            id: id.clone(),
            option: option.clone(),
            result: None,
            waker: None,
            completed: false,
        }
    }

    pub(crate) fn result(&mut self) -> Result<Option<Value>, Error> {
        self.result.take().unwrap()
    }

    pub(crate) fn complete(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }

    fn is_completed(&self) -> bool {
        self.completed
    }

    fn set_waker(&mut self, waker: Waker) {
        self.completed = false;
        self.waker = Some(waker);
    }
}

#[allow(dead_code)]
pub(crate) struct FindPeerCmd {
    id: Id,
    expected_seq: i32,
    option: LookupOption,

    result: Option<Result<Vec<Peer>, Error>>,
    waker: Option<Waker>,
    completed: bool,
}

#[allow(dead_code)]
impl FindPeerCmd {
    pub(crate) fn new(id: &Id, expected_seq: i32, option: &LookupOption) -> Self {
        Self {
            id: id.clone(),
            expected_seq,
            option: option.clone(),
            result: None,
            waker: None,
            completed: false,
        }
    }

    pub(crate) fn result(&mut self) -> Result<Vec<Peer>, Error> {
        self.result.take().unwrap()
    }

    pub(crate) fn complete(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
            self.completed = true;
        }
    }

    fn is_completed(&self) -> bool {
        self.completed
    }

    fn set_waker(&mut self, waker: Waker) {
        self.completed = false;
        self.waker = Some(waker);
    }
}

#[allow(dead_code)]
pub(crate) struct StoreValueCmd {
    value: Value,

    result: Option<Result<(), Error>>,

    waker: Option<Waker>,
    completed: bool,
}

#[allow(dead_code)]
impl StoreValueCmd {
    pub(crate) fn new(value: &Value) -> Self {
        Self {
            value: value.clone(),
            result: None,
            waker: None,
            completed: false,
        }
    }

    pub(crate) fn result(&mut self) -> Result<(), Error> {
        self.result.take().unwrap()
    }

    pub(crate) fn complete(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
            self.completed = true;
        }
    }

    fn is_completed(&self) -> bool {
        self.completed
    }

    fn set_waker(&mut self, waker: Waker) {
        self.completed = false;
        self.waker = Some(waker);
    }
}

#[allow(dead_code)]
pub(crate) struct AnnouncePeerCmd {
    peer: Peer,

    result: Option<Result<(), Error>>,

    waker: Option<Waker>,
    completed: bool,
}

#[allow(dead_code)]
impl AnnouncePeerCmd {
    pub(crate) fn new(peer: &Peer) -> Self {
        Self {
            peer: peer.clone(),
            result: None,
            waker: None,
            completed: false,
        }
    }

    pub(crate) fn result(&mut self) -> Result<(), Error> {
        self.result.take().unwrap()
    }

    pub(crate) fn complete(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
            self.completed = true;
        }
    }

    fn is_completed(&self) -> bool {
        self.completed
    }

    fn set_waker(&mut self, waker: Waker) {
        self.completed = false;
        self.waker = Some(waker);
    }
}

#[derive(Clone)]
pub(crate) enum Command {
    FindNode(Arc<Mutex<FindNodeCmd>>),
    FindValue(Arc<Mutex<FindValueCmd>>),
    FindPeer(Arc<Mutex<FindPeerCmd>>),
    StoreValue(Arc<Mutex<StoreValueCmd>>),
    AnnouncePeer(Arc<Mutex<AnnouncePeerCmd>>)
}

impl Command {
    pub(crate) fn is_completed(&self) -> bool {
        match self {
            Command::FindNode(c) => c.lock().unwrap().is_completed(),
            Command::FindValue(c) => c.lock().unwrap().is_completed(),
            Command::FindPeer(c) => c.lock().unwrap().is_completed(),
            Command::StoreValue(c) => c.lock().unwrap().is_completed(),
            Command::AnnouncePeer(c) => c.lock().unwrap().is_completed(),
        }
    }

    fn set_waker(&mut self, waker: Waker) {
        match self {
            Command::FindNode(s) => s.lock().unwrap().set_waker(waker),
            Command::FindValue(s) => s.lock().unwrap().set_waker(waker),
            Command::FindPeer(s) => s.lock().unwrap().set_waker(waker),
            Command::StoreValue(s) => s.lock().unwrap().set_waker(waker),
            Command::AnnouncePeer(s) => s.lock().unwrap().set_waker(waker),
        }
    }
}

pub(crate) struct CmdFuture {
    command: Command,
}

impl CmdFuture {
    pub(crate) fn new(cmd: Command) -> Self {
        CmdFuture {
            command: cmd,
        }
    }
}

impl Future for CmdFuture {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.command.is_completed() {
            Poll::Ready(Ok(()))
        } else {
            self.command.set_waker(cx.waker().clone());
            Poll::Pending
        }
    }
}
