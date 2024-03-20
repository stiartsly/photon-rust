
use crate::node_info::NodeInfo;

pub(crate) struct BootstrapZone {
    nodes: Vec<Box<NodeInfo>>,
    updated: bool,
}

#[allow(dead_code)]
impl BootstrapZone {
    pub(crate) fn from(input: &[NodeInfo]) -> Self {
        let mut bs = Self {
            nodes: Vec::new(),
            updated: false,
        };
        bs.push_many(input);
        bs
    }

    pub(crate) fn push(&mut self, node: &NodeInfo) {
        self.nodes.push(Box::new(node.clone()));
        self.updated = true;
    }

    pub(crate) fn push_many(&mut self, nodes: &[NodeInfo]) {
        nodes.iter().for_each(|item| {
            self.push(item);
        });
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
