use std::collections::HashMap;
use std::time::SystemTime;

use crate::{
    as_millis,
    cryptobox::{CryptoBox, KeyPair, Nonce, PublicKey},
    error::Error,
    id::Id,
};

pub(crate) const EXPIRED_CHECK_INTERVAL: u64 = 60 * 1000;
pub(crate) struct CryptoCache {
    keypair: KeyPair,
    cache: HashMap<Id, Entry>,
}

#[allow(dead_code)]
impl CryptoCache {
    pub(crate) fn new(keypair: &KeyPair) -> CryptoCache {
        CryptoCache {
            keypair: keypair.clone(),
            cache: HashMap::new(),
        }
    }

    pub(crate) fn get(&mut self, key: &Id) -> &Box<CryptoContext> {
        if self.cache.get(key).is_none() {
            let entry = Entry::new(self.load(key));
            self.cache.insert(key.clone(), entry);
        }

        let entry = self.cache.get_mut(key).unwrap();
        &entry.0
    }

    pub(crate) fn handle_expiration(&self) {
        //unimplemented!()
        // TODO;
    }

    fn load(&self, key: &Id) -> Box<CryptoContext> {
        Box::new(CryptoContext::new(&key.to_encryption_key(), &self.keypair))
    }

    fn on_removal(&self, _: &Box<CryptoContext>) {
        // Don't need to close in native.
        // val.close();
    }
}

struct Entry(Box<CryptoContext>, SystemTime);

#[allow(dead_code)]
impl Entry {
    fn new(value: Box<CryptoContext>) -> Self {
        Entry(value, SystemTime::now())
    }

    fn expired(&self) -> bool {
        as_millis!(&self.1) >= EXPIRED_CHECK_INTERVAL as u128
    }
}

#[allow(dead_code)]
pub(crate) struct CryptoContext {
    box_: CryptoBox,
    nonce: Nonce,
}

impl CryptoContext {
    fn new(pk: &PublicKey, keypair: &KeyPair) -> CryptoContext {
        let recver = Id::from_bytes(pk.as_bytes());
        let sender = Id::from_bytes(keypair.public_key().as_bytes());
        let distance = Id::distance(&sender, &recver);

        CryptoContext {
            box_: CryptoBox::try_from(pk, keypair.private_key()).unwrap(),
            nonce: Nonce::from(distance.as_bytes()),
        }
    }

    pub(crate) fn encrypt(&self, plain: &[u8], cipher: &mut [u8]) -> Result<(), Error> {
        self.box_.encrypt(plain, cipher, &self.nonce)
    }

    pub(crate) fn decrypt(&self, cipher: &[u8], plain: &mut [u8]) -> Result<(), Error> {
        self.box_.decrypt(cipher, plain, &self.nonce)
    }

    pub(crate) fn encrypt_into(&self, plain: &[u8]) -> Vec<u8> {
        self.box_.encrypt_into(plain, &self.nonce)
    }

    pub(crate) fn decrypt_into(&self, cipher: &[u8]) -> Vec<u8> {
        self.box_.decrypt_into(cipher, &self.nonce)
    }
}
