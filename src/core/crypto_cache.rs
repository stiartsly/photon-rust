use std::collections::HashMap;
use std::time::SystemTime;

use crate::{
    as_millis,
    Id,
    cryptobox::{CryptoBox, KeyPair, Nonce, PublicKey},
    error::Error,
};

pub(crate) const EXPIRED_CHECK_INTERVAL: u64 = 60 * 1000;
pub(crate) struct CryptoCache {
    keypair: KeyPair,
    cache: HashMap<Id, Entry>,
}

#[allow(dead_code)]
impl CryptoCache {
    pub(crate) fn new(keypair: KeyPair) -> CryptoCache {
        CryptoCache {
            keypair,
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
        println!("handle expiration for cryptocontext");
        // TODO;
    }

    fn load(&self, key: &Id) -> Box<CryptoContext> {
        Box::new(CryptoContext::new(&key.to_encryption_pubkey(), &self.keypair))
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

pub(crate) struct CryptoContext {
    cbox: CryptoBox,
    nonce: Nonce,
}

impl CryptoContext {
    fn new(pk: &PublicKey, keypair: &KeyPair) -> CryptoContext {
        let recver = Id::from_bytes(pk.as_bytes());
        let sender = Id::from_encryption_pubkey(keypair.public_key());
        let distance = Id::distance(&sender, &recver);

        CryptoContext {
            cbox: CryptoBox::try_from(pk, keypair.private_key()).unwrap(),
            nonce: Nonce::from(&distance.as_bytes()[0..Nonce::BYTES]),
        }
    }

    pub(crate) fn encrypt_into(&self, plain: &[u8]) -> Result<Vec<u8>, Error> {
        self.cbox.encrypt_into(plain, &self.nonce)
    }

    pub(crate) fn decrypt_into(&self, cipher: &[u8]) -> Result<Vec<u8>, Error> {
        self.cbox.decrypt_into(cipher, &self.nonce)
    }
}
