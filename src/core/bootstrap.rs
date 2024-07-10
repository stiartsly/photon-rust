
use crate::node_info::NodeInfo;

pub(crate) struct BootstrapZone {
    nodes: Vec<Box<NodeInfo>>,
    updated: bool,
}

#[allow(dead_code)]
impl BootstrapZone {
    pub(crate) fn from(input: Vec<NodeInfo>) -> Self {
        let mut bs = Self {
            nodes: Vec::new(),
            updated: false,
        };
        bs.push_many(input);
        bs
    }

    pub(crate) fn push(&mut self, node: NodeInfo) {
        self.nodes.push(Box::new(node));
        self.updated = true;
    }

    pub(crate) fn push_many(&mut self, mut nodes: Vec<NodeInfo>) {
        while let Some(item) = nodes.pop() {
            self.push(item)
        }
        self.updated = true;
    }

    pub(crate) fn pop_all<F>(&mut self, mut f: F) where F: FnMut(Box<NodeInfo>) {
        if !self.updated {
            return;
        }

        while let Some(item) = self.nodes.pop() {
            f(item);
        }

        self.updated = false;
        assert!(self.nodes.is_empty());
    }
}
