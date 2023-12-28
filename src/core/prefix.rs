use crate::id::{Id, ID_BITS};

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

    pub fn depth(&self) -> i32 {
        self.depth
    }

    pub fn is_splittable(&self) -> bool {
        self.depth < (ID_BITS - 1) as i32
    }

    pub fn first(&self) -> Id {
        self.id.clone()
    }

    pub fn last(&self) -> Id {
        Id::new()
    }

    pub fn parent(&self) -> Prefix {
        Prefix::new()
    }

    pub fn split_branch(&self, _: bool) -> Prefix {
        Prefix::new()
    }

    pub fn is_sibling_of(&self, _: &Prefix) -> bool {
        false
    }

    pub fn create_random_id(&self) -> Id {
        Id::random()
    }
}
