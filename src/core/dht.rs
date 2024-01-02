
use std::net::SocketAddr;

#[allow(dead_code)]
pub(crate) struct DHT {
    persist_root: String,
}


pub(crate) trait Protocols {

}

#[allow(dead_code)]
impl DHT {
    pub(crate) fn new(_: &SocketAddr) -> Self {
        DHT {
            persist_root: "".to_string(),
        }
    }

    pub(crate) fn enable_persistence(&mut self, path: &str) {
        self.persist_root = path.to_string()
    }
}