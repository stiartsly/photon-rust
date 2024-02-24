use crate::{
    id::Id,
    cryptobox::KeyPair,
    error::Error
};

#[allow(dead_code)]
pub(crate) struct CryptoCache {
    keypair: KeyPair
}

#[allow(dead_code)]
impl CryptoCache {
    pub(crate) fn new(keypair: &KeyPair) -> CryptoCache {
        CryptoCache {
            keypair: keypair.clone(),
        }
    }

    pub(crate) fn get(&self, _: &Id) -> &CryptoContext {
        unimplemented!()
    }
}

#[allow(dead_code)]
pub(crate) struct CryptoContext {
}

impl CryptoContext {
    pub(crate) fn encrypt(&self, _: &[u8], _: &mut [u8]) -> Result<(), Error> {
        unimplemented!()
    }

    pub(crate) fn decrypt(&self, _: &[u8], _: &mut [u8]) -> Result<(), Error> {
        unimplemented!()
    }

    pub(crate) fn encrypt_into(&self, _: &[u8]) -> Vec<u8> {
        unimplemented!()
    }

    pub(crate) fn decrypt_into(&self, _: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}