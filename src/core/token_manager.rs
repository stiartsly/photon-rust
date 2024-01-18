use std::net::SocketAddr;
use crate::id::Id;

#[allow(dead_code)]
pub(crate) struct TokenManager {
}

#[allow(dead_code)]
impl TokenManager {
    pub(crate) fn new() -> Self {
        TokenManager {}
    }

    pub(crate) fn generate_token(&self) -> i32 {
        unimplemented!()
    }

    pub(crate) fn verify_token(&self, _: i32, _: &Id, _: &SocketAddr, _: &Id) -> bool {
        unimplemented!()
    }
}
