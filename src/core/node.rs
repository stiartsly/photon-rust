use crate::{
    signature::KeyPair,
    id::Id
};

struct Node {
    key_pair: KeyPair,
    id: Id,

    persistent:bool
}

impl Node {
    pub fn new() -> Self {
        Node {
            key_pair: KeyPair::random(),
            id: Id::random(),
            persistent: false
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}
