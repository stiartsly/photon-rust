use crate::id::{Id, ID_BYTES};
use crate::signature::{self, KeyPair, PrivateKey, Signature};
use crate::unwrap;
use std::fmt;
use std::mem;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug)]
pub struct Peer {
    pk: Id,
    sk: Option<PrivateKey>,
    id: Id,
    origin: Id,
    port: u16,
    url: Option<String>,
    sig: Vec<u8>,
}

pub struct Builder<'a> {
    keypair: Option<KeyPair>,
    id: &'a Id,
    origin: Option<&'a Id>,
    port: u16,
    url: Option<&'a str>,
}

impl<'a> Builder<'a> {
    pub fn default(node_id: &'a Id) -> Self {
        Builder {
            keypair: None,
            id: node_id,
            origin: None,
            port: 0,
            url: None,
        }
    }
    pub fn with_keypair(&mut self, keypair: &'a KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone());
        self
    }

    pub fn with_origin(&mut self, origin: &'a Id) -> &mut Self {
        self.origin = Some(origin);
        self
    }

    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn with_alternative_url(&mut self, alternative_url: &'a str) -> &mut Self {
        self.url = Some(alternative_url);
        self
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
        let kp = unwrap!(b.keypair);
        let mut peer = Peer {
            pk: Id::from_signature_key(kp.public_key()),
            sk: Some(kp.private_key().clone()),
            id: b.id.clone(),
            origin: match b.origin {
                Some(origin) => origin.clone(),
                None => b.id.clone(),
            },
            port: b.port,
            url: match b.url {
                Some(url) => Some(url.nfc().collect::<String>()),
                None => None,
            },
            sig: Vec::new(),
        };

        let sig = signature::sign(peer.serialize_signature_data().as_slice(), unwrap!(peer.sk));
        peer.sig = sig;
        peer
    }

    #[allow(dead_code)]
    fn pack(_: &Builder) -> Self {
        unimplemented!()
    }

    pub const fn id(&self) -> &Id {
        &self.pk
    }

    pub const fn has_private_key(&self) -> bool {
        self.sk.is_some()
    }

    pub const fn private_key(&self) -> Option<&PrivateKey> {
        self.sk.as_ref()
    }

    pub const fn node_id(&self) -> &Id {
        &self.id
    }

    pub const fn origin(&self) -> &Id {
        &self.origin
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub const fn has_alternative_url(&self) -> bool {
        self.url.is_some()
    }

    pub const fn alternative_url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    pub fn signature(&self) -> &[u8] {
        &self.sig
    }

    pub fn is_delegated(&self) -> bool {
        self.origin != self.id
    }

    pub fn is_valid(&self) -> bool {
        assert_eq!(
            self.sig.len(),
            Signature::BYTES,
            "Invalid signature data length {}, should be {}",
            self.sig.len(),
            Signature::BYTES
        );

        let sigdata = self.serialize_signature_data();
        let pk = self.pk.to_signature_key();
        signature::verify(sigdata.as_ref(), self.sig.as_slice(), &pk)
    }

    fn serialize_signature_data(&self) -> Vec<u8> {
        let mut len: usize = 0;

        len += ID_BYTES * 2;
        len += mem::size_of::<u16>(); // padding port

        if self.url.is_some() {
            len += unwrap!(self.url).len();
        }

        let mut input = Vec::<u8>::with_capacity(len);
        input.extend_from_slice(self.id.as_bytes());
        input.extend_from_slice(self.origin.as_bytes());
        input.extend_from_slice(self.port.to_le_bytes().as_ref());

        if self.url.is_some() {
            input.extend_from_slice(unwrap!(self.url).as_ref());
        }
        input
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},", self.pk, self.id)?;
        if self.is_delegated() {
            write!(f, "{},", self.origin)?;
        }
        write!(f, "{}", self.port)?;
        if self.url.is_some() {
            write!(f, ",{}", unwrap!(self.url))?;
        }
        Ok(())
    }
}
