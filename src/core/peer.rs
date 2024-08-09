use std::fmt;
use std::mem;
use unicode_normalization::UnicodeNormalization;

use crate::{
    unwrap,
    id::{Id, ID_BYTES},
    signature::{self, KeyPair, PrivateKey, Signature}
};

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

pub(crate) struct PackBuilder<'a> {
    pk: Option<Id>,
    node_id: Option<Id>,
    port: u16,
    url: Option<&'a str>,
    sig: Option<Vec<u8>>,
}

impl<'a> PackBuilder<'a> {
    pub(crate) fn new() -> Self {
        Self {
            pk: None,
            node_id: None,
            port: 0,
            url: None,
            sig: None,
        }
    }

    pub(crate) fn with_peerid(&mut self, peerid: Id) -> &mut Self {
        self.pk = Some(peerid);
        self
    }

    pub(crate) fn with_nodeid(&mut self, proxyid: Id) -> &mut Self {
        self.node_id = Some(proxyid);
        self
    }

    pub(crate) fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub(crate) fn with_alternative_url(&mut self, alternative_url: &'a str) -> &mut Self {
        self.url = Some(alternative_url);
        self
    }

    pub(crate) fn with_sigature(&mut self, sig: &[u8]) -> &mut Self {
        self.sig = Some(sig.to_vec());
        self
    }

    pub fn build(&mut self) -> Peer {
        Peer::packed(self)
    }
}

#[derive(Clone, Debug)]
pub struct Peer {
    pk: Id,
    sk: Option<PrivateKey>,
    id: Id,
    origin: Option<Id>,
    port: u16,
    url: Option<String>,
    sig: Vec<u8>,
}

impl Peer {
    fn new(b: &Builder) -> Self {
        let kp = unwrap!(b.keypair);
        let mut peer = Peer {
            pk: Id::from_signature_pubkey(kp.public_key()),
            sk: Some(kp.private_key().clone()),
            id: b.id.clone(),
            origin: b.origin.map(|v|v.clone()),
            port: b.port,
            url: b.url.map(|v| v.nfc().collect::<String>()),
            sig: Vec::new(),
        };

        peer.sig = signature::sign(
            peer.serialize_signature_data().as_slice(),
            unwrap!(peer.sk)
        );
        peer
    }

    pub(crate) fn packed(b: &mut PackBuilder) -> Self {
        // TODO:
        Peer {
            pk: b.pk.take().unwrap(),
            sk: None,
            id: b.node_id.take().unwrap(),
            origin: None,
            port: b.port,
            url: match b.url {
                Some(url) => Some(url.nfc().collect::<String>()),
                None => None,
            },
            sig: Vec::new(),
        }
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

    pub fn origin(&self) -> &Id {
        self.origin.as_ref().unwrap_or_else(|| self.node_id())
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
        self.origin.is_some()
    }

    pub fn is_valid(&self) -> bool {
        assert_eq!(
            self.sig.len(),
            Signature::BYTES,
            "Invalid signature data length {}, should be {}",
            self.sig.len(),
            Signature::BYTES
        );

        signature::verify(
            self.serialize_signature_data().as_ref(),
            self.sig.as_slice(),
            &self.pk.to_signature_pubkey()
        )
    }

    fn serialize_signature_data(&self) -> Vec<u8> {
        let mut len = 0usize;

        len += ID_BYTES * 2;
        len += mem::size_of::<u16>(); // padding port
        len += match self.url.as_ref() {
            Some(url) => url.len(),
            None => 0,
        };

        let mut data = vec![0u8; len];

        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(self.origin().as_bytes());
        data.extend_from_slice(self.port.to_le_bytes().as_ref());

        if let Some(url) = self.url.as_ref() {
            data.extend_from_slice(url.as_ref());
        }
        data
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},", self.pk, self.id)?;
        if self.is_delegated() {
            write!(f, "{},", self.origin())?;
        }
        write!(f, "{}", self.port)?;
        if self.url.is_some() {
            write!(f, ",{}", unwrap!(self.url))?;
        }
        Ok(())
    }
}
