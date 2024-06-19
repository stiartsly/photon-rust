
use crate::node_info::NodeInfo;

pub(crate) struct Data {
    nodes4: Option<Vec<NodeInfo>>,
    nodes6: Option<Vec<NodeInfo>>,
    token: i32,
}

impl Data {
    pub(crate) fn new() -> Self {
        Self {
            nodes4: None,
            nodes6: None,
            token: 0,
        }
    }
}

pub(crate) trait Msg {
    fn data(&self) -> &Data;
    fn data_mut(&mut self) -> &mut Data;

    fn nodes4(&self) -> Option<&[NodeInfo]> {
        self.data().nodes4.as_deref()
    }

    fn nodes6(&self) -> Option<&[NodeInfo]> {
        self.data().nodes6.as_deref()
    }

    fn token(&self) -> i32 {
        self.data().token
    }

    fn populate_closest_nodes4(&mut self, nodes: Vec<NodeInfo>) {
        self.data_mut().nodes4 = Some(nodes)
    }

    fn populate_closest_nodes6(&mut self, nodes: Vec<NodeInfo>) {
        self.data_mut().nodes6 = Some(nodes)
    }

    fn populate_token(&mut self, token: i32) {
        self.data_mut().token = token
    }
}
