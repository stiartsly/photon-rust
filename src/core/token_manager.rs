use std::net::SocketAddr;
use std::time::SystemTime;
use crate::id::Id;

const TOKEN_TIMEOUT: u128 = 5 * 60 * 1000;   //5 minutes

macro_rules! as_millis {
    ($time:expr) => {{
        $time.elapsed().unwrap().as_millis()
    }};
}

#[allow(dead_code)]
pub(crate) struct TokenManager {
    session_secret: [u8; 32],
    timestamp: SystemTime,
    previous_timestamp: SystemTime
}

#[allow(dead_code)]
impl TokenManager {
    pub(crate) fn new() -> Self {
        let mut seed = [0u8; 32];
        unsafe {
            libsodium_sys::randombytes_buf(
                seed.as_mut_ptr() as *mut libc::c_void,
                32
            ); // Always success.
        }
        TokenManager {
            session_secret: seed,
            timestamp: SystemTime::UNIX_EPOCH,
            previous_timestamp: SystemTime::UNIX_EPOCH
        }
    }

    fn update_token_timestamp(&mut self) {
        while as_millis!(&self.timestamp) > TOKEN_TIMEOUT {
            self.previous_timestamp = self.timestamp;
            self.timestamp = SystemTime::now();
            break;
        }
    }

    pub(crate) fn generate_token(&self, nodeid: &Id, addr: &SocketAddr, target: &Id) -> i32 {
        generate_token(nodeid, addr, target, &self.timestamp, &self.session_secret )
    }

    pub(crate) fn verify_token(&mut self, token: i32, nodeid: &Id, addr: &SocketAddr, target: &Id) -> bool {
        self.update_token_timestamp();

        let current = generate_token(nodeid, addr, target, &self.timestamp, &self.session_secret);
        match token == current {
            true => { return true },
            false => {}
        }

        let prev = generate_token(nodeid, addr, target, &self.previous_timestamp, &self.session_secret);
        token == prev
    }
}

fn generate_token(_: &Id, _: &SocketAddr, _: &Id, _: &SystemTime, _: &[u8]) -> i32 {
    /*let port:u16 = addr.port();

    let mut input: Vec<u8> = Vec::new();
    input.extend_from_slice(nodeid.as_vec());
    input.extend_from_slice(port.to_le_bytes().as_ref());
    input.extend_from_slice(target.as_vec());
    input.extend_from_slice(as_millis!(timestamp).to_le_bytes().as_ref());
    input.extend_from_slice(session_secret);
    digest(input)*/
    unimplemented!()
}
