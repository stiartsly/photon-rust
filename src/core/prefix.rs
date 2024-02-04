use std::fmt;
use crate::id;
use crate::id::Id;

#[derive(Clone, PartialEq, Eq)]
pub struct Prefix {
    depth: i32,
    id: Id,
}

impl Prefix {
    pub fn new() -> Self {
        Prefix {
            id: Id::default(),
            depth: -1
        }
    }

    pub const fn id(&self) -> &Id {
        &self.id
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
        let p = Prefix {
            id: Id::max(),
            depth: self.depth
        };
        let d = p.id.distance(&Id::max());
        self.id.distance(&d)
    }

    pub fn parent(&self) -> Prefix {
        let mut parent = self.clone();
        if self.depth == -1 {
            return parent;
        }

        // set last bit to zero
        parent.set_tail(parent.depth);
        parent.depth -= 1;
        parent
    }

    pub fn split_branch(&mut self, high_branch: bool) -> Prefix {
        let mut branch = self.clone();
        let depth = branch.depth as usize;
        branch.depth += 1;

        let bytes = self.id.as_mut_bytes();
        let val = 0x80 >> (depth % 8);
        if high_branch {
            bytes[depth / 8] |= val;
        } else {
            bytes[depth / 8] &= !val;
        }
        branch
    }

    pub fn is_sibling_of(&self, other: &Prefix) -> bool {
        self.depth == other.depth &&
            id::bits_equal(&self.id, &other.id, self.depth-1)
    }

    pub fn create_random_id(&self) -> Id {
        let mut id = Id::random();
        id::bits_copy(&self.id, &mut id, self.depth);
        id
    }

    fn set_tail(&mut self, bit: i32) {
        let index = (bit >> 3) as usize;
        let bytes = self.id.as_mut_bytes();
        bytes[index] &= !(0x80 >> (bit & 0x07))
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.depth == -1 {
            write!(f, "all")?;
        }

        let end_index = ((self.depth + 8) >> 3) as usize;
        let slice = self.id.as_bytes()[..end_index].to_vec();

        write!(f,
            "{}/{}",
            hex::encode(slice),
            self.depth
        )?;

        Ok(())
    }
}
