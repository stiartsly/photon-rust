use crate::msg::error_msg::ErrorMsg;

use super::{
    ping_req::PingRequestMsg,
    ping_rsp::PingResponseMsg,
    find_node_req::FindNodeRequestMsg,
    find_node_rsp::FindNodeResponseMsg,
    find_value_req::FindValueRequestMsg,
    find_value_rsp::FindValueResponseMsg,
    find_peer_req::FindPeerRequestMsg,
    find_peer_rsp::FindPeerResponseMsg,
    store_value_req::StoreValueRequestMsg,
    store_value_rsp::StoreValueResponseMsg,
    announce_peer_req::AnnouncePeerRequestMsg,
    announce_peer_rsp::AnnouncePeerResponseMsg
};

#[allow(dead_code)]
pub(crate) enum Message {
    Error(ErrorMsg),
    PingRequest(PingRequestMsg),
    PingResponse(PingResponseMsg),
    FindNodeRequest(FindNodeRequestMsg),
    FindNodeResponse(FindNodeResponseMsg),
    FindValueRequest(FindValueRequestMsg),
    FindValueResponse(FindValueResponseMsg),
    FindPeerRequest(FindPeerRequestMsg),
    FindPeerResponse(FindPeerResponseMsg),
    StoreValueREquest(StoreValueRequestMsg),
    StoreValueResponse(StoreValueResponseMsg),
    AnnouncePeerRequest(AnnouncePeerRequestMsg),
    AnnouncePeerResponse(AnnouncePeerResponseMsg)
}

