use crate::id::{Id, ID_BYTES};
use crate::{
    signature,
    cryptobox,
    unwrap
};

use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Value {
    pk: Option<Id>,
    sk: Option<signature::PrivateKey>,
    recipient: Option<Id>,
    nonce: Option<cryptobox::Nonce>,
    sig: Option<Vec<u8>>,
    data: Vec<u8>,
    seq: i32
}

pub struct ValueBuilder<'a> {
    data:&'a [u8]
}

pub struct SignedBuidler<'a> {
    keypair: Option<signature::KeyPair>,
    nonce: Option<cryptobox::Nonce>,

    data: &'a [u8],
    seq: i32
}

pub struct EncryptedBuilder<'a> {
    keypair: Option<signature::KeyPair>,
    nonce: Option<cryptobox::Nonce>,

    recipient: &'a Id,
    data: &'a [u8],
    seq: i32
}

#[allow(dead_code)]
pub(crate) struct PackBuilder {
    pk: Option<Id>,
    recipient: Option<Id>,
    nonce: Option<cryptobox::Nonce>,
    sig: Option<Vec<u8>>,
    data: Vec<u8>,
    seq: i32
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
        SignedBuidler {
            data: value,
            keypair: None,
            nonce: None,
            seq: 0
        }
    }

    pub fn with_keypair(&mut self, keypair: &signature::KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone()); self
    }

    pub fn with_nonce(&mut self, nonce: &cryptobox::Nonce) -> &mut Self {
        self.nonce = Some(nonce.clone()); self
    }

    pub fn with_sequence_number(&mut self, sequence_number: i32) -> &mut Self {
        self.seq = sequence_number; self
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
        EncryptedBuilder {
            data: value,
            keypair: None,
            nonce: None,
            seq: 0,
            recipient
        }
    }

    pub fn with_keypair(&mut self, keypair: &signature::KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone()); self
    }

    pub fn with_nonce(&mut self, nonce: &cryptobox::Nonce) -> &mut Self {
        self.nonce = Some(nonce.clone()); self
    }

    pub fn with_sequence_number(&mut self, sequence_number: i32) -> &mut Self {
        self.seq = sequence_number; self
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

impl Value {
    fn default(b: &ValueBuilder) -> Value {
        Value {
            pk: None,
            sk: None,
            recipient: None,
            nonce: None,
            sig: None,
            data: b.data.to_vec(),
            seq: -1
        }
    }

    fn signed(b: &SignedBuidler) -> Value {
        assert!(b.keypair.is_some());
        assert!(b.nonce.is_some());

        let kp = unwrap!(b.keypair);
        let mut value = Value {
            pk: Some(Id::from_signature_key(kp.public_key())),
            sk: Some(kp.private_key().clone()),
            recipient: None,
            nonce: Some(b.nonce.unwrap().clone()),
            sig: None,
            data: b.data.to_vec(),
            seq: b.seq,
        };
        let sig = signature::sign(
            value.to_signdata().as_slice(),
            unwrap!(value.sk)
        );
        value.sig = Some(sig);
        value
    }

    fn encrypted(b: &EncryptedBuilder) -> Value {
        assert!(b.keypair.is_some());
        assert!(b.nonce.is_some());

        let kp = unwrap!(b.keypair);
        let mut value = Value {
            pk: Some(Id::from_signature_key(kp.public_key())),
            sk: Some(kp.private_key().clone()),
            recipient: Some(b.recipient.clone()),
            nonce: Some(b.nonce.unwrap().clone()),
            sig: None,
            data: b.data.to_vec(),
            seq: b.seq
        };

        let owner_sk = cryptobox::PrivateKey::from_signature_key(
            kp.private_key()
        );

        value.data = cryptobox::encrypt_into(
            b.data,
            unwrap!(b.nonce),
            &b.recipient.to_encryption_key(),
            &owner_sk.unwrap());

        let sig = signature::sign(
            value.to_signdata().as_slice(),
            unwrap!(value.sk),
        );
        value.sig = Some(sig);
        value
    }

    #[allow(dead_code)]
    fn pack(_: &PackBuilder) -> Self {
        unimplemented!()
    }

    pub fn id(&self) -> Id {
        let mut input: Vec<u8> = Vec::new();
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

    pub fn public_key(&self) -> Option<&Id> {
        self.pk.as_ref()
    }

    pub fn recipient(&self) -> Option<&Id> {
        self.recipient.as_ref()
    }

    pub fn has_private_key(&self) -> bool {
        self.sk.is_some()
    }

    pub fn private_key(&self) -> Option<&signature::PrivateKey> {
        self.sk.as_ref()
    }

    pub fn sequence_number(&self) -> i32 {
        self.seq
    }

    pub fn nonce(&self) -> Option<&cryptobox::Nonce> {
        self.nonce.as_ref()
    }

    pub fn signature(&self) -> Option<&[u8]> {
        match self.sig.as_ref() {
            Some(s) => Some(s.as_slice()),
            None => None
        }
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn size(&self) -> usize {
        let mut len = self.data.len();
        match self.sig.as_ref() {
            Some(sig) => len += sig.len(),
            None => {}
        }
        len
    }

    pub fn is_encrypted(&self) -> bool {
        self.recipient.is_some()
    }

    pub fn is_signed(&self) -> bool {
        self.sig.is_some()
    }

    pub fn is_mutable(&self) -> bool {
        self.pk.is_some()
    }

    pub fn is_valid(&self) -> bool {
        assert!(!self.data.is_empty());

        if !self.is_mutable() {
            return true;
        }

        assert!(self.sig.is_some());
        assert!(self.pk.is_some());

        signature::verify(
            self.to_signdata().as_slice(),
            unwrap!(self.sig).as_slice(),
            &unwrap!(self.pk).to_signature_key()
        )
    }

    fn to_signdata(&self) -> Vec<u8> {
        let mut len = 0;

        len += match self.is_encrypted() {
            true => { ID_BYTES },
            false => { 0 }
        };
        len += cryptobox::Nonce::BYTES;
        len += std::mem::size_of::<i32>();
        len += self.data.len();

        let mut input:Vec<u8> = Vec::with_capacity(len);
        if self.is_encrypted() {
            input.extend_from_slice(unwrap!(self.recipient).as_bytes());
        }
        input.extend_from_slice(unwrap!(self.nonce).as_bytes());
        input.extend_from_slice(self.seq.to_le_bytes().as_ref());
        input.extend_from_slice(self.data.as_ref());

        input
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id:{}", self.id())?;
        if self.is_mutable() {
            write!(f,
                ",publicKey:{}, nonce:{}",
                unwrap!(self.pk),
                unwrap!(self.nonce)
            )?;
        }
        if self.is_encrypted() {
            write!(f,
                ",recipient:{}",
                unwrap!(self.recipient)
            )?;
        }
        if self.is_signed() {
            write!(f,
                ",sig:{}",
                hex::encode(unwrap!(self.sig))
            )?;
        }
        write!(f,
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
