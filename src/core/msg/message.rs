use crate::msg::error::ErrorMsg;

use super::ping_request::PingRequestMsg;
use super::ping_response::PingResponseMsg;

enum Method {

}

pub(crate) enum Message {
    Error(ErrorMsg),
    PingRequest(PingRequestMsg),
    PingResponse(PingResponseMsg)
}

