use std::option::Option;
use std::fmt;
use crate::id::{Id, ID_BYTES};
use crate::signature::{PrivateKey, KeyPair, Signature};

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
    pub fn new(id: &Id, port: u16) -> Result<PeerInfo, &'static str> {
        let key_pair = KeyPair::random();
        PeerInfo::with_all(&key_pair, id, id, port, &"".to_string())
    }

    pub fn with_key_pair(key_pair: &KeyPair, id: &Id, port: u16) -> Self {
        PeerInfo {
            public_key: Id::from_signature_key(key_pair.public_key()),
            private_key: Some(*key_pair.private_key()),
            node_id: *id,
            origin: *id,
            port,
            alternative_url: None,
            signature: Vec::new()
        }
    }

    fn with_all(key_pair: &KeyPair, node_id: &Id, origin: &Id, port: u16, alternative_url: &String)
        -> Result<PeerInfo, &'static str> {
        if port == 0 {
            return Err("Invalid port value");
        }

        let public_key = Id::from_signature_key(key_pair.public_key());
        let private_key = Some(key_pair.private_key().clone());
        let node_id = node_id.clone();
        let origin = origin.clone();
        let alternative_url = match alternative_url.is_empty() {
            true => None,
            false => Some(alternative_url.clone())
        };

        Ok(PeerInfo {
            public_key,
            private_key,
            node_id,
            origin,
            port,
            alternative_url,
            signature: Vec::new(),
        })
    }

    pub fn id(&self) -> &Id {
        &self.public_key
    }

    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    pub fn private_key(&self) -> Result<&PrivateKey, &'static str> {
        match self.private_key.as_ref() {
            Some(pk) => { Ok(pk) }
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
            Some(url) => { Ok(url) },
            None => {
                Err("No binding alternative url")
            }
        }
    }

    pub fn signature(&self) -> &Vec<u8> {
        &self.signature
    }

    pub fn is_delegated(&self) -> bool {
        self.node_id != self.origin
    }

    pub fn is_valid(&self) -> bool {
        if self.signature.len() != Signature::BYTES {
            return false
        }

        let capacity = self.fill_sign_data_size();
        let mut data = vec![0u8; capacity];
        self.fill_sign_data(&mut data);

        let pk = self.public_key.to_signature_key();
        match Signature::verify(data.as_ref(), self.signature.as_slice(), &pk) {
            Ok(valid) => { valid },
            Err(_) => {false}
        }
    }

    fn fill_sign_data<'a>(&self, _: &'a mut [u8]) {
        // TODO:
    }

    fn fill_sign_data_size(&self) -> usize {
        let mut size = ID_BYTES * 2 + std::mem::size_of::<u16>();
        if self.has_alternative_url() {
            size += self.alternative_url.as_deref().unwrap().len();
        }
        return size;
    }
}

impl fmt::Display for PeerInfo {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
