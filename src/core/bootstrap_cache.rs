
use crate::node_info::NodeInfo;

pub(crate) struct BootstrapCache {
    nodes: Vec<Box<NodeInfo>>,
    updated: bool,
}

#[allow(dead_code)]
impl BootstrapCache {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            updated: false,
        }
    }

    pub(crate) fn push(&mut self, node: &NodeInfo) {
        self.nodes.push(Box::new(node.clone()));
        self.updated = true;
    }

    pub(crate) fn push_many(&mut self, nodes: &[NodeInfo]) {
        nodes.iter().for_each(|item| {
            self.nodes.push(Box::new(item.clone()));
        });
        self.updated = true;
    }

    pub(crate) fn pop_all<F>(&mut self, mut f: F)
    where F: FnMut(Box<NodeInfo>) {
        if !self.updated {
            assert!(self.nodes.is_empty());
            return;
        }

        while let Some(item) = self.nodes.pop() {
            f(item);
        }

        self.updated = false;
    }
}
