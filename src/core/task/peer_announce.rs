use std::fmt;
use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;
use log::error;

use crate::{
    Peer,
    dht::DHT,
};

use crate::msg::{
    announce_peer_req,
};

use super::{
    closest_set::ClosestSet,
    candidate_node::CandidateNode,
    task::{Task, TaskData},
};

#[allow(dead_code)]
pub(crate) struct PeerAnnounceTask {
    base_data: TaskData,

    todo: Rc<RefCell<LinkedList<Rc<RefCell<CandidateNode>>>>>,
    peer: Option<Rc<Peer>>,
}

#[allow(dead_code)]
impl PeerAnnounceTask {
    pub(crate) fn new(dht: Rc<RefCell<DHT>>, closest: &ClosestSet, peer: &Rc<Peer>) -> Self {
        let mut todo = LinkedList::new();
        for item in closest.entries() {
            todo.push_back(item);
        }

        Self {
            base_data: TaskData::new(dht),
            todo: Rc::new(RefCell::new(todo)),
            peer: Some(peer.clone()),
        }
    }
}

impl Task for PeerAnnounceTask {
    fn data(&self) -> &TaskData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut TaskData {
        &mut self.base_data
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self) {
        while self.can_request() {
            let cn = {
                match self.todo.borrow().front() {
                    Some(cn) => cn.clone(),
                    None => break,
                }
            };

            let msg = Rc::new(RefCell::new(announce_peer_req::Message::new()));
            let cloned_todo = self.todo.clone();
            if let Err(err) = self.send_call(cn, msg, Box::new(move|_| {
                cloned_todo.borrow_mut().pop_front();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }
    }
}

impl fmt::Display for PeerAnnounceTask {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
