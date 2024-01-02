
use std::fmt;
use crate::{id, id::Id};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Prefix {
    depth: i32,
    id: Id,
}

impl Prefix {
    pub fn new() -> Self {
        Prefix {
            depth: -1,
            id: Id::zero(),
        }
    }

    pub const fn depth(&self) -> i32 {
        self.depth
    }

    pub const fn is_splittable(&self) -> bool {
        self.depth < (id::ID_BITS - 1) as i32
    }

    pub fn first(&self) -> Id {
        self.id.clone()
    }

    pub fn last(&self) -> Id {
        unimplemented!();
    }

    pub fn parent(&self) -> Prefix {
        unimplemented!();
    }

    pub fn split_branch(&self, _: bool) -> Prefix {
        unimplemented!();
    }

    pub fn is_sibling_of(&self, _: &Prefix) -> bool {
        unimplemented!();
    }

    pub fn create_random_id(&self) -> Id {
        let mut id = Id::random();
        id::bits_copy(&self.id, &mut id, self.depth);
        return id;
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}
