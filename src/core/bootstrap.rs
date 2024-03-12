use crate::node_info::NodeInfo;

pub(crate) struct Bootstrap {
    nodes: Vec<NodeInfo>,
    updated: bool,
}

#[allow(dead_code)]
impl Bootstrap {
    pub(crate) fn from(input: &[NodeInfo]) -> Self {
        let mut nodes = Vec::new() as Vec<NodeInfo>;
        input.iter().for_each(|item| {
            nodes.push(item.clone());
        });
        Self {
            nodes,
            updated: true,
        }
    }

    pub(crate) fn add_many(&mut self, nodes: &[NodeInfo]) {
        nodes.iter().for_each(|item| {
            self.nodes.push(item.clone());
        });
        self.updated = true;
    }

    pub(crate) fn add_one(&mut self, node: &NodeInfo) {
        self.nodes.push(node.clone());
        self.updated = true;
    }

    pub(crate) fn update<F>(&mut self, mut f: F) where F: FnMut(&[NodeInfo]) {
        f(self.nodes.as_ref());
        self.updated = false;
    }
}
