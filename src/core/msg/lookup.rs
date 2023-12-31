
use crate::id::Id;
use crate::node_info::NodeInfo;

pub(crate) struct RequestField {
    target: Id,
    want4: bool,
    want6: bool,
    want_token: bool
}

pub(crate) struct ResponseFields {
    node4: Vec<NodeInfo>,
    node6: Vec<NodeInfo>,
    token: i32
}

impl RequestField {
    pub(crate) fn new() -> Self {
        RequestField {
            target: Id::random(),
            want4: false,
            want6: false,
            want_token: false
        }
    }
}

impl ResponseFields {
    pub(crate) fn new() -> Self {
        ResponseFields {
            node4: Vec::new(),
            node6: Vec::new(),
            token: 0
        }
    }

}