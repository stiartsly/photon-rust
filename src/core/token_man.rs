use std::net::{SocketAddr, IpAddr};
use std::time::SystemTime;
use sha2::{Digest, Sha256};

use crate::id::Id;
use crate::{as_millis, as_usize};

const TOKEN_TIMEOUT: u128 = 5 * 60 * 1000;   //5 minutes

pub(crate) struct TokenManager {
    session_secret: [u8; 32],
    timestamp: SystemTime,
    previous_timestamp: SystemTime
}

impl TokenManager {
    pub(crate) fn new() -> Self {
        let mut seed = [0u8; 32];
        unsafe { // Always success.
            libsodium_sys::randombytes_buf(
                seed.as_mut_ptr() as *mut libc::c_void,
                32
            );
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

        let current = generate_token(
            nodeid,
            addr,
            target,
            &self.timestamp,
            &self.session_secret
        );

        match token == current {
            true => { return true },
            false => {}
        }

        let prev = generate_token(nodeid,
            addr,
            target,
            &self.previous_timestamp,
            &self.session_secret
        );
        token == prev
    }
}

fn generate_token(nodeid: &Id, addr: &SocketAddr, target: &Id, timestamp: &SystemTime, secret: &[u8]) -> i32 {
    let mut input: Vec<u8> = Vec::new();
    let port:u16 = addr.port();

    input.extend_from_slice(nodeid.as_bytes());
    input.extend_from_slice(port.to_le_bytes().as_ref());
    input.extend_from_slice(target.as_bytes());

    match addr.ip() {
        IpAddr::V4(ipv4) => input.extend_from_slice(ipv4.octets().as_ref()),
        IpAddr::V6(ipv6) => input.extend_from_slice(ipv6.octets().as_ref())
    };
    input.extend_from_slice(as_millis!(timestamp).to_le_bytes().as_ref());
    input.extend_from_slice(secret);

    let mut hasher = Sha256::new();
    hasher.update(input);
    let digest = hasher.finalize().to_vec();

    let pos = (as_usize!(digest[0]) & 0xff) & 0x1f ; // mod 32
    let token = ((as_usize!(digest[pos]) & 0xff) << 24) |
            ((as_usize!(digest[(pos + 1) & 0x1f]) & 0xff) << 16) |
            ((as_usize!(digest[(pos + 2) & 0x1f]) & 0xff) << 8) |
            (as_usize!(digest[(pos + 3) & 0x1f]) & 0xff);

    token as i32
}
