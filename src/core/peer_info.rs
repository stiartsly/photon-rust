use std::option::Option;
use crate::id::Id;
use crate::signature::{PrivateKey, KeyPair};

#[derive(Debug)]
pub struct PeerInfo {
    public_key: Id,
    private_key: Option<PrivateKey>,
    node_id: Id,
    origin: Id,
    port: u16,
    alternative_url: Option<String>,
    signature: Vec<u8>,
}

impl PeerInfo {
    pub fn new(id: &Id, port: u16) -> Self {
        PeerInfo {
            public_key: *id,
            private_key: None,
            node_id: *id,
            origin: *id,
            port,
            alternative_url: None,
            signature: Vec::new()
        }
    }

    pub fn new_with_keypair(keypair: &KeyPair, id: &Id, port: u16) -> Self {
        PeerInfo {
            public_key: *id,
            private_key: Some(keypair.private_key()),
            node_id: *id,
            origin: *id,
            port,
            alternative_url: None,
            signature: Vec::new()
        }
    }

    pub fn id(&self) -> &Id {
        &self.public_key
    }

    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    pub fn private_key(&self) -> Result<&PrivateKey, &'static str> {
        match self.private_key.as_ref() {
            Some(pk) => {
                Ok(pk)
            },
            None => {
                Err("No binding private key")
            }
        }
    }

    pub fn node_id(&self) -> &Id {
        &self.node_id
    }

    pub fn origin(&self) -> &Id {
        &self.origin
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn has_alternative_url(&self) -> bool {
        self.alternative_url.is_some()
    }

    pub fn alternative_url(&self) -> Result<&String, &'static str> {
        match self.alternative_url.as_ref() {
            Some(url) => {
                Ok(url)
            },
            None => {
                Err("No binding alternative url")
            }
        }
    }

    pub fn signature(&self) -> &Vec<u8> {
        &self.signature
    }

    pub fn is_delegated(&self) -> bool {
        self.node_id == self.origin
    }

    pub fn is_valid(&self) -> bool {
        // TODO;
        false
    }
}
