use crate::id::Id;
use crate::signature;
use crate::cryptobox;
use crate::error::Error;

#[allow(dead_code)]
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
        ValueBuilder { data: value }
    }

    pub fn build(&self) -> Result<Value, Error> {
        if self.data.is_empty() {
            return Err(Error::Argument(format!("Value can not be empty")));
        }
        Ok(Value::default(self))
    }
}

impl<'a> SignedValueBuidler<'a> {
    pub fn default(value: &'a [u8]) -> Self {
        SignedValueBuidler {
            data: value,
            keypair: None,
            nonce: None,
            seq: 0
        }
    }

    pub fn with_keypair(&mut self, keypair: &'a signature::KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone()); self
    }

    pub fn with_nonce(&mut self, nonce: &'a cryptobox::Nonce) -> &mut Self {
        self.nonce = Some(nonce.clone()); self
    }

    pub fn with_sequence_number(&mut self, sequence_number: i32) -> &mut Self {
        self.seq = sequence_number; self
    }

    pub fn buld(&mut self) -> Result<Value, Error> {
        if self.data.is_empty() {
            return Err(Error::Argument(format!("Value can not be empty")));
        }
        if self.keypair.is_none() {
            self.keypair = Some(signature::KeyPair::random());
        }
        if self.nonce.is_none() {
            self.nonce = Some(cryptobox::Nonce::random());
        }

        Ok(Value::with_signed(self))
    }
}

impl<'a> EncryptedValueBuidler<'a> {
    pub fn default(value: &'a [u8], recipient: &'a Id) -> Self {
        EncryptedValueBuidler {
            data: value,
            keypair: None,
            nonce: None,
            recipient,
            seq: 0
        }
    }

    pub fn with_keypair(&mut self, keypair: &'a signature::KeyPair) -> &mut Self {
        self.keypair = Some(keypair.clone()); self
    }

    pub fn with_nonce(&mut self, nonce: &'a cryptobox::Nonce) -> &mut Self {
        self.nonce = Some(nonce.clone()); self
    }

    pub fn with_sequence_number(&mut self, sequence_number: i32) -> &mut Self {
        self.seq = sequence_number; self
    }

    pub fn buld(&mut self) -> Result<Value, Error> {
        if self.data.is_empty() {
            return Err(Error::Argument(format!("Value can not be empty")));
        }
        if self.keypair.is_none() {
            self.keypair = Some(signature::KeyPair::random());
        }
        if self.nonce.is_none() {
            self.nonce = Some(cryptobox::Nonce::random());
        }

        Ok(Value::with_encrypted(self))
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
        // TODO: signature.
        Value {
            pk: Some(Id::from_signature_key(b.keypair.unwrap().public_key())),
            sk: Some(b.keypair.unwrap().private_key().clone()),
            recipent: None,
            nonce: Some(b.nonce.unwrap().clone()),
            signature: None,
            data: b.data.to_vec(),
            seq: b.seq,
        }
    }

    fn with_encrypted(b: &EncryptedValueBuidler) -> Value {
        // TODO: signature.
        Value {
            pk: Some(Id::from_signature_key(b.keypair.unwrap().public_key())),
            sk: Some(b.keypair.unwrap().private_key().clone()),
            recipent: Some(b.recipient.clone()),
            nonce: Some(b.nonce.unwrap().clone()),
            signature: None,
            data: b.data.to_vec(),
            seq: b.seq
        }
    }

    pub fn id(&self) -> Id {
        unimplemented!()
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

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    pub fn signature(&self) -> Option<&[u8]> {
        match self.signature.as_ref() {
            Some(s) => Some(&s[..]),
            None => None
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
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
        unimplemented!()
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

pub fn value_id(_: &Value) -> Id {
    unimplemented!()
}
