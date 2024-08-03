
use crate::node_info::NodeInfo;

pub(crate) struct BootstrapChannel {
    nodes: Vec<NodeInfo>,
    updated: bool,
}

#[allow(dead_code)]
impl BootstrapChannel {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            updated: false,
        }
    }

    pub(crate) fn push(&mut self, node: &NodeInfo) {
        self.nodes.push(node.clone());
        self.updated = true;
    }

    pub(crate) fn push_many(&mut self, nodes: &[NodeInfo]) {
        nodes.iter().for_each(|item| {
            self.nodes.push(item.clone());
        });
        self.updated = true;
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn pop_all<F>(&mut self, mut f: F)
    where F: FnMut(NodeInfo) {
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
