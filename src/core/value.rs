use crate::id::{self, Id};
use crate::signature;
use crate::cryptobox;

use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct Value {
    pk: Option<Id>,
    sk: Option<signature::PrivateKey>,
    recipent: Option<Id>,
    nonce: Option<cryptobox::Nonce>,
    signature: Option<Vec<u8>>,
    data: Vec<u8>,
    seq: i32
}

pub struct ValueBuilder<'a> {
    data:&'a [u8]
}

pub struct SignedValueBuidler<'a> {
    keypair: Option<signature::KeyPair>,
    nonce: Option<cryptobox::Nonce>,

    data: &'a [u8],
    seq: i32
}

pub struct EncryptedValueBuidler<'a> {
    keypair: Option<signature::KeyPair>,
    nonce: Option<cryptobox::Nonce>,

    recipient: &'a Id,
    data: &'a [u8],
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

impl<'a> SignedValueBuidler<'a> {
    pub fn default(value: &'a [u8]) -> Self {
        assert!(!value.is_empty(), "Value data can not be empty");
        SignedValueBuidler {
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

        Value::with_signed(self)
    }
}

impl<'a> EncryptedValueBuidler<'a> {
    pub fn default(value: &'a [u8], recipient: &'a Id) -> Self {
        assert!(!value.is_empty(), "Value data can not be empty");
        EncryptedValueBuidler {
            data: value,
            keypair: None,
            nonce: None,
            recipient,
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

        Value::with_encrypted(self)
    }
}

impl Value {
    fn default(b: &ValueBuilder) -> Value {
        Value {
            pk: None,
            sk: None,
            recipent: None,
            nonce: None,
            signature: None,
            data: b.data.to_vec(),
            seq: -1
        }
    }

    fn with_signed(b: &SignedValueBuidler) -> Value {
        let mut value = Value {
            pk: Some(Id::from_signature_key(b.keypair.unwrap().public_key())),
            sk: Some(b.keypair.unwrap().private_key().clone()),
            recipent: None,
            nonce: Some(b.nonce.unwrap().clone()),
            signature: None,
            data: b.data.to_vec(),
            seq: b.seq,
        };
        let signature = signature::sign(
            value.to_signdata().as_slice(),
            value.sk.as_ref().unwrap()
        );
        value.signature = Some(signature);
        value
    }

    fn with_encrypted(b: &EncryptedValueBuidler) -> Value {
        let mut value = Value {
            pk: Some(Id::from_signature_key(b.keypair.unwrap().public_key())),
            sk: Some(b.keypair.unwrap().private_key().clone()),
            recipent: Some(b.recipient.clone()),
            nonce: Some(b.nonce.unwrap().clone()),
            signature: None,
            data: b.data.to_vec(),
            seq: b.seq
        };

        let owner_sk = cryptobox::PrivateKey::from_signature_key(
            b.keypair.unwrap().private_key()
        );

        value.data = cryptobox::encrypt_into(
            b.data,
            b.nonce.as_ref().unwrap(),
            &b.recipient.to_encryption_key(),
            &owner_sk.unwrap());

        let signature = signature::sign(
            value.to_signdata().as_slice(),
            value.sk.as_ref().unwrap()
        );
        value.signature = Some(signature);
        value
    }

    pub fn id(&self) -> Id {
        let mut input: Vec<u8> = Vec::new();
        match self.pk.as_ref() {
            Some(pk) => {
                input.extend_from_slice(pk.as_bytes());
                input.extend_from_slice(self.nonce.as_ref().unwrap().as_bytes());
            },
            None => {
                input.extend_from_slice(self.data.as_ref())
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(input);
        Id::try_from_bytes(hasher.finalize().as_slice())
    }

    pub fn public_key(&self) -> Option<&Id> {
        self.pk.as_ref()
    }

    pub fn recipient(&self) -> Option<&Id> {
        self.recipent.as_ref()
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
        match self.signature.as_ref() {
            Some(s) => Some(&s[..]),
            None => None
        }
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn size(&self) -> usize {
        let mut len = self.data.len();
        match self.signature.as_ref() {
            Some(sig) => len += sig.len(),
            None => {}
        }
        len
    }

    pub fn is_encrypted(&self) -> bool {
        self.recipent.is_some()
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    pub fn is_mutable(&self) -> bool {
        self.pk.is_some()
    }

    pub fn is_valid(&self) -> bool {
        assert!(!self.data.is_empty(), "data should not be empty");

        match self.is_mutable() {
            true => signature::verify(
                self.to_signdata().as_slice(),
                self.signature.as_ref().unwrap().as_slice(),
                &self.public_key().unwrap().to_signature_key()
            ),
            false => true
        }
    }

    fn to_signdata(&self) -> Vec<u8> {
        let mut len = 0;

        len += match self.is_encrypted() {
            true => { id::ID_BYTES },
            false => { 0 }
        };
        len += cryptobox::Nonce::BYTES;
        len += std::mem::size_of::<i32>();
        len += self.data.len();

        let mut input:Vec<u8> = Vec::with_capacity(len);
        if self.is_encrypted() {
            input.extend_from_slice(self.recipent.unwrap().as_bytes());
        }
        input.extend_from_slice(self.nonce.unwrap().as_bytes());
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
                self.pk.as_ref().unwrap(),
                self.nonce.as_ref().unwrap()
            )?;
        }
        if self.is_encrypted() {
            write!(f,
                ",recipient:{}",
                self.recipent.as_ref().unwrap()
            )?;
        }
        if self.is_signed() {
            write!(f,
                ",sig:{}",
                hex::encode(self.signature.as_ref().unwrap())
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
