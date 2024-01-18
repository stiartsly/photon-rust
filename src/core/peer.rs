use unicode_normalization::UnicodeNormalization;

use crate::id::{Id, ID_BYTES};
use crate::signature::{PrivateKey, KeyPair, Signature};
use crate::error::Error;

#[derive(Debug)]
pub struct Peer {
    pk: Id,
    sk: Option<PrivateKey>,
    id: Id,
    origin: Option<Id>,
    port: u16,
    url: Option<String>,
    signature: Vec<u8>,
}

#[allow(dead_code)]
pub struct Builder<'a> {
    keypair: Option<KeyPair>,
    id: &'a Id,
    origin: Option<&'a Id>,
    port: u16,
    url: Option<&'a str>,
    signature: Option<&'a [u8]>
}

impl<'a> Builder<'a> {
    pub fn default(node_id: &'a Id) -> Self {
        Builder {
            keypair: None,
            id: node_id,
            origin: None,
            port: 0,
            url: None,
            signature: None
        }
    }
    pub fn with_keypair(&mut self, keypair: &'a KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone()); self
    }

    pub fn with_origin(&mut self, origin: &'a Id) -> &mut Self {
        self.origin = Some(origin); self
    }

    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port; self
    }

    pub fn with_alternative_url(&mut self, alternative_url: &'a str) -> &mut Self {
        self.url = Some(alternative_url); self
    }

    pub fn build(&mut self) -> Peer {
        if self.keypair.is_none() {
            self.keypair = Some(KeyPair::random())
        }
        Peer::new(self)
    }
}

impl Peer {
    fn new(b: &Builder) -> Self {
        Peer {
            pk: Id::from_signature_key(b.keypair.unwrap().public_key()),
            sk: Some(*b.keypair.unwrap().private_key()),
            id: b.id.clone(),
            origin: match b.origin {
                Some(origin) => Some (origin.clone()),
                None => None
            },
            port: b.port,
            url: match b.url {
                Some(url) => {Some(url.nfc().collect::<String>())},
                None => None,
            },
            signature: Vec::new()
        }
    }

    pub fn id(&self) -> &Id {
        &self.pk
    }

    pub fn has_private_key(&self) -> bool {
        self.sk.is_some()
    }

    pub fn private_key(&self) -> Option<&PrivateKey> {
        self.sk.as_ref()
    }

    pub fn node_id(&self) -> &Id {
        &self.id
    }

    pub fn has_origin(&self) -> bool {
        self.origin.is_some()
    }

    pub fn origin(&self) -> Option<&Id> {
        self.origin.as_ref()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn has_alternative_url(&self) -> bool {
        self.url.is_some()
    }

    pub fn alternative_url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    pub fn signature(&self) -> &Vec<u8> {
        &self.signature
    }

    pub fn is_delegated(&self) -> bool {
        self.origin.is_some() && self.origin.unwrap() != self.id
    }

    pub fn is_valid(&self) -> Result<(), Error> {
        if self.signature.len() != Signature::BYTES {
            return Err(Error::State(format!("Invalid signature data length")));
        }

        let capacity = self.fill_sign_data_size();
        let mut data = vec![0u8; capacity];
        self.fill_sign_data(&mut data);

        let pk = self.pk.to_signature_key();
        match Signature::verify(data.as_ref(), self.signature.as_slice(), &pk) {
            Ok(_) => {Ok(())},
            Err(_) => {
                Err(Error::Crypto(format!("Bad signature value")))
            }
        }
    }

    fn fill_sign_data<'a>(&self, _: &'a mut [u8]) {
        unimplemented!()
    }

    fn fill_sign_data_size(&self) -> usize {
        let mut size = ID_BYTES * 2 + std::mem::size_of::<u16>();
        if self.url.is_some() {
            size += self.url.as_deref().unwrap().len();
        }
        return size;
    }
}

impl std::fmt::Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},", self.pk, self.id)?;
        if self.is_delegated() {
            write!(f, "{},", self.origin.unwrap())?;
        }
        write!(f, "{}", self.port)?;
        if self.url.is_some() {
            write!(f, ",{}", self.url.as_ref().unwrap())?;
        }
        Ok(())
    }
}
