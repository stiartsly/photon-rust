use std::fmt;
use ciborium::value::Value as CVal;
use sha2::{Digest, Sha256};

use crate::{
    unwrap,
    cryptobox,
    signature,
    id::{Id, ID_BYTES},
    error::Error,
};

#[derive(Clone, Debug)]
pub struct Value {
    pk: Option<Id>,
    sk: Option<signature::PrivateKey>,
    recipient: Option<Id>,
    nonce: Option<cryptobox::Nonce>,
    sig: Option<Vec<u8>>,
    data: Vec<u8>,
    seq: i32,
}

pub struct ValueBuilder<'a> {
    data: &'a [u8],
}

pub struct SignedBuidler<'a> {
    keypair: Option<signature::KeyPair>,
    nonce: Option<cryptobox::Nonce>,

    data: &'a [u8],
    seq: i32,
}

pub struct EncryptedBuilder<'a> {
    keypair: Option<signature::KeyPair>,
    nonce: Option<cryptobox::Nonce>,

    recipient: &'a Id,
    data: &'a [u8],
    seq: i32,
}

pub(crate) struct PackBuilder<'a> {
    pk: Option<Id>,
    recipient: Option<Id>,
    nonce: Option<cryptobox::Nonce>,
    sig: Option<&'a [u8]>,
    data: &'a [u8],
    seq: i32,
}

impl<'a> ValueBuilder<'a> {
    pub fn default(value: &'a [u8]) -> Self {
        assert!(!value.is_empty(), "Value data can not be empty");
        ValueBuilder { data: value }
    }

    pub fn build(&self) -> Value {
        Value::default(self)
    }
}

impl<'a> SignedBuidler<'a> {
    pub fn default(value: &'a [u8]) -> Self {
        assert!(!value.is_empty(), "Value data can not be empty");
        Self {
            data: value,
            keypair: None,
            nonce: None,
            seq: 0,
        }
    }

    pub fn with_keypair(&mut self, keypair: &signature::KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone());
        self
    }

    pub fn with_nonce(&mut self, nonce: &cryptobox::Nonce) -> &mut Self {
        self.nonce = Some(nonce.clone());
        self
    }

    pub fn with_sequence_number(&mut self, sequence_number: i32) -> &mut Self {
        self.seq = sequence_number;
        self
    }

    pub fn buld(&mut self) -> Value {
        if self.keypair.is_none() {
            self.keypair = Some(signature::KeyPair::random());
        }
        if self.nonce.is_none() {
            self.nonce = Some(cryptobox::Nonce::random());
        }

        Value::signed(self)
    }
}

impl<'a> EncryptedBuilder<'a> {
    pub fn default(value: &'a [u8], recipient: &'a Id) -> Self {
        assert!(!value.is_empty(), "Value data can not be empty");
        Self {
            data: value,
            keypair: None,
            nonce: None,
            seq: 0,
            recipient,
        }
    }

    pub fn with_keypair(&mut self, keypair: &signature::KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone());
        self
    }

    pub fn with_nonce(&mut self, nonce: &cryptobox::Nonce) -> &mut Self {
        self.nonce = Some(nonce.clone());
        self
    }

    pub fn with_sequence_number(&mut self, sequence_number: i32) -> &mut Self {
        self.seq = sequence_number;
        self
    }

    pub fn buld(&mut self) -> Value {
        if self.keypair.is_none() {
            self.keypair = Some(signature::KeyPair::random());
        }
        if self.nonce.is_none() {
            self.nonce = Some(cryptobox::Nonce::random());
        }

        Value::encrypted(self)
    }
}

#[allow(dead_code)]
impl<'a> PackBuilder<'a> {
    pub(crate) fn default(value: &'a [u8]) -> Self {
        Self {
            pk: None,
            recipient: None,
            nonce: None,
            sig: None,
            data: value,
            seq: -1,
        }
    }

    pub(crate) fn with_pk(&mut self, pk: Id) -> &mut Self {
        self.pk = Some(pk);
        self
    }

    pub(crate) fn with_recipient(&mut self, recipient: Id) -> &mut Self {
        self.recipient = Some(recipient);
        self
    }

    pub(crate) fn with_nonce(&mut self, nonce: &'a [u8]) -> &mut Self {
        self.nonce = Some(cryptobox::Nonce::from(nonce));
        self
    }

    pub(crate) fn with_sig(&mut self, sig: &'a [u8]) -> &mut Self {
        self.sig = Some(sig);
        self
    }

    pub(crate) fn with_seq(&mut self, seq: i32) -> &mut Self {
        self.seq = seq;
        self
    }

    pub(crate) fn build(&mut
        self) -> Value {
        Value::packed(self)
    }
}

impl Value {
    fn default(b: &ValueBuilder) -> Value {
        Self {
            pk: None,
            sk: None,
            recipient: None,
            nonce: None,
            sig: None,
            data: b.data.to_vec(),
            seq: -1,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn try_from_cbor(_: &Value) -> Result<Self, Error> {
        unimplemented!()
    }

    fn signed(b: &SignedBuidler) -> Value {
        assert!(b.keypair.is_some());
        assert!(b.nonce.is_some());

        let kp = unwrap!(b.keypair);
        let mut value = Value {
            pk: Some(Id::from_signature_pubkey(kp.public_key())),
            sk: Some(kp.private_key().clone()),
            recipient: None,
            nonce: Some(unwrap!(b.nonce).clone()),
            sig: None,
            data: b.data.to_vec(),
            seq: b.seq,
        };
        let sig = signature::sign(
            value.serialize_signature_data().as_slice(),
            unwrap!(value.sk),
        );
        value.sig = Some(sig);
        value
    }

    fn encrypted(b: &EncryptedBuilder) -> Value {
        assert!(b.keypair.is_some());
        assert!(b.nonce.is_some());

        let kp = unwrap!(b.keypair);
        let mut value = Value {
            pk: Some(Id::from_signature_pubkey(kp.public_key())),
            sk: Some(kp.private_key().clone()),
            recipient: Some(b.recipient.clone()),
            nonce: Some(unwrap!(b.nonce).clone()),
            sig: None,
            data: b.data.to_vec(),
            seq: b.seq,
        };

        let owner_sk = cryptobox::PrivateKey::from_signature_key(kp.private_key());

        value.data = cryptobox::encrypt_into(
            b.data,
            unwrap!(b.nonce),
            &b.recipient.to_encryption_pubkey(),
            &owner_sk.unwrap(),
        ).ok().unwrap();

        let sig = signature::sign(
            value.serialize_signature_data().as_slice(),
            unwrap!(value.sk),
        );
        value.sig = Some(sig);
        value
    }

    fn packed(b: &mut PackBuilder) -> Self {
        Value {
            pk: b.pk.take(),
            sk: None,
            recipient: b.recipient.take(),
            nonce: b.nonce.take(),
            sig: b.sig.map(|v| v.to_vec()),
            data: b.data.to_vec(),
            seq: b.seq,
        }
    }

    pub fn id(&self) -> Id {
        let mut input = Vec::new();
        match self.pk.as_ref() {
            Some(pk) => {
                input.extend_from_slice(pk.as_bytes());
                input.extend_from_slice(unwrap!(self.nonce).as_bytes());
            },
            None => {
                input.extend_from_slice(self.data.as_slice())
            }
        }

        let mut sha256 = Sha256::new();
        sha256.update(input);
        Id::from_bytes(sha256.finalize().as_slice())
    }

    pub const fn public_key(&self) -> Option<&Id> {
        self.pk.as_ref()
    }

    pub const fn recipient(&self) -> Option<&Id> {
        self.recipient.as_ref()
    }

    pub const fn has_private_key(&self) -> bool {
        self.sk.is_some()
    }

    pub const fn private_key(&self) -> Option<&signature::PrivateKey> {
        self.sk.as_ref()
    }

    pub const fn sequence_number(&self) -> i32 {
        self.seq
    }

    pub const fn nonce(&self) -> Option<&cryptobox::Nonce> {
        self.nonce.as_ref()
    }

    pub fn signature(&self) -> Option<&[u8]> {
        match self.sig.as_ref() {
            Some(s) => Some(s.as_slice()),
            None => None,
        }
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn size(&self) -> usize {
        let mut len = self.data.len();
        if let Some(sig) = self.sig.as_ref() {
            len += sig.len();
        }
        len
    }

    pub const fn is_encrypted(&self) -> bool {
        self.recipient.is_some()
    }

    pub const fn is_signed(&self) -> bool {
        self.sig.is_some()
    }

    pub const fn is_mutable(&self) -> bool {
        self.pk.is_some()
    }

    pub fn is_valid(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        if !self.is_mutable() {
            return true;
        }

        assert!(self.sig.is_some());
        assert!(self.pk.is_some());

        signature::verify(
            self.serialize_signature_data().as_slice(),
            unwrap!(self.sig).as_slice(),
            &unwrap!(self.pk).to_signature_pubkey(),
        )
    }

    fn serialize_signature_data(&self) -> Vec<u8> {
        let mut len = 0;

        len += match self.is_encrypted() {
            true => ID_BYTES,
            false => 0,
        };
        len += cryptobox::Nonce::BYTES;
        len += std::mem::size_of::<i32>();
        len += self.data.len();

        let mut input = Vec::with_capacity(len);
        if self.is_encrypted() {
            input.extend_from_slice(unwrap!(self.recipient).as_bytes());
        }
        input.extend_from_slice(unwrap!(self.nonce).as_bytes());
        input.extend_from_slice(self.seq.to_le_bytes().as_ref());
        input.extend_from_slice(self.data.as_ref());

        input
    }

    pub(crate) fn to_cbor(&self) -> CVal {
        unimplemented!()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id:{}", self.id())?;
        if self.is_mutable() {
            write!(
                f,
                ",publicKey:{}, nonce:{}",
                unwrap!(self.pk),
                unwrap!(self.nonce)
            )?;
        }
        if self.is_encrypted() {
            write!(f, ",recipient:{}", unwrap!(self.recipient))?;
        }
        if self.is_signed() {
            write!(f, ",sig:{}", hex::encode(unwrap!(self.sig)))?;
        }
        write!(
            f,
            "seq:{}, data:{}",
            self.seq,
            hex::encode(self.data.as_slice())
        )?;
        Ok(())
    }
}

pub fn value_id(value: &Value) -> Id {
    value.id()
}
