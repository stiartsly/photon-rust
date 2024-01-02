use crate::msg::error_msg::ErrorMsg;

use super::ping_req::PingRequestMsg;
use super::ping_rsp::PingResponseMsg;

#[allow(dead_code)]
pub(crate) enum Message {
    Error(ErrorMsg),
    PingRequest(PingRequestMsg),
    PingResponse(PingResponseMsg)
}

