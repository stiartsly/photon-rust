use std::any::Any;
use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;
// use log::error;

use crate::{
    peer::Peer,
};

// use crate::msg::{
//    announce_peer_req,
//    msg::Msg,
// };

use super::{
    closest_set::ClosestSet,
    candidate_node::CandidateNode,
    task::{Task, TaskData},
};

#[allow(dead_code)]
pub(crate) struct PeerAnnounceTask {
    base_data: TaskData,

    // TODO: todo: Rc<RefCell<LinkedList<Rc<RefCell<KBucketEntry>>>>>,
    todo: Rc<RefCell<LinkedList<Rc<RefCell<CandidateNode>>>>>,
    peer: Option<Box<Peer>>,
}

#[allow(dead_code)]
impl PeerAnnounceTask {
    pub(crate) fn new(closest: &ClosestSet, peer: &Peer) -> Self {
        let mut todo = LinkedList::new();
        for item in closest.entries() {
            todo.push_back(item);
        }

        Self {
            base_data: TaskData::new(),
            todo: Rc::new(RefCell::new(todo)),
            peer: Some(Box::new(peer.clone())),
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
        /*
        while !self.todo.borrow().is_empty() && self.can_request() {
            let candidate_node = match self.todo.borrow().front() {
                Some(node) => node,
                None => break,
            };

            let req = Rc::new(RefCell::new(announce_peer_req::Message::new()));

            let cloned = Rc::clone(candidate_node);
            let cloned_todo = Rc::clone(&self.todo);
            if let Err(err) = self.send_call(cloned, req, Box::new(move|_| {
                cloned_todo.borrow_mut().pop_front();
            })) {
               error!("Error on sending 'findNode' message: {:?}", err);
            }
        }*/
        unimplemented!()
    }
}
