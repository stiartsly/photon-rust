use std::fmt;
use static_assertions::const_assert;
use libsodium_sys::{
    crypto_box_SECRETKEYBYTES,
    crypto_box_PUBLICKEYBYTES,
    crypto_box_NONCEBYTES,
    crypto_box_SEEDBYTES,
    crypto_box_BEFORENMBYTES,
    crypto_box_MACBYTES,
    crypto_sign_ed25519_sk_to_curve25519,
    crypto_sign_ed25519_pk_to_curve25519
};

use crate::signature;
use crate::error::Error;

const_assert!(PrivateKey::BYTES == crypto_box_SECRETKEYBYTES as usize);
const_assert!(PublicKey::BYTES == crypto_box_PUBLICKEYBYTES as usize);
const_assert!(Nonce::BYTES == crypto_box_NONCEBYTES as usize);
const_assert!(KeyPair::SEED_BYTES == crypto_box_SEEDBYTES as usize);
const_assert!(CryptoBox::SYMMETRIC_KEY_BYTES == crypto_box_BEFORENMBYTES as usize);
const_assert!(CryptoBox::MAC_BYTES == crypto_box_MACBYTES as usize);

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PrivateKey {
    key: [u8; Self::BYTES]
}

impl PrivateKey {
    pub const BYTES: usize = 32;

    pub fn new() -> Self {
        PrivateKey {
            key: [0; Self::BYTES]
        }
    }

    pub fn from(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            Self::BYTES,
            "Incorrect raw private key size {}, should be {}",
            input.len(),
            Self::BYTES
        );

        let sk: [u8; Self::BYTES] = input.try_into().unwrap();
        PrivateKey { key: sk }
    }

    pub fn from_signature_key(sign_sk: &signature::PrivateKey) -> Self {
        let mut bytes = [0u8; Self::BYTES];
        unsafe { // Always success.
            crypto_sign_ed25519_sk_to_curve25519(
                bytes.as_mut_ptr() as *mut libc::c_uchar,
                sign_sk.as_bytes().as_ptr() as *mut libc::c_uchar
            );
        }
        PrivateKey { key: bytes }
    }

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.key.as_ref()
    }

    pub fn clear(&mut self) {
        self.key.fill(0);
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.key))?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PublicKey {
    key: [u8; Self::BYTES]
}

impl PublicKey {
    pub const BYTES: usize = 32;

    pub fn new() -> Self {
        PublicKey { key: [0; Self::BYTES] }
    }

    pub fn from(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            Self::BYTES,
            "Incorrect raw public key size {}, should be {}",
            input.len(),
            Self::BYTES
        );

        let sk: [u8; Self::BYTES] = input.try_into().unwrap();
        PublicKey { key: sk }
    }

    pub fn from_signature_key(sign_pk: &signature::PublicKey) -> Result<Self, Error> {
        let mut bytes = [0u8; Self::BYTES];
        unsafe { // Always success.
            let result = crypto_sign_ed25519_pk_to_curve25519(
                bytes.as_mut_ptr() as *mut libc::c_uchar,
                sign_pk.as_bytes().as_ptr() as *mut libc::c_uchar
            );

            if result != 0 {
                return Err(Error::Crypto(format!("converts Ed25519 key to x25519 key failed.")));
            }
        }
        Ok(PublicKey { key: bytes })
    }

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.key.as_ref()
    }

    pub fn clear(&mut self) {
        self.key.fill(0);
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.key))?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Nonce {
    nonce: [u8; Self::BYTES]
}

impl Nonce {
    pub const BYTES: usize = 24;

    pub fn new() -> Self {
        Nonce {
            nonce: [0; Self::BYTES]
        }
    }
    pub fn random() -> Self {
        unimplemented!()
    }

    pub fn as_bytes(&self) -> &[u8] {
        unimplemented!()
    }

    pub fn clear(&mut self) {
        self.nonce.fill(0)
    }
}

impl std::fmt::Display for Nonce {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct KeyPair {
    sk: PrivateKey,
    pk: PublicKey
}

impl KeyPair {
    pub const SEED_BYTES: usize = 32;

    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn from_private_key(_: &PrivateKey) -> Self {
        unimplemented!()
    }

    pub fn from_seed(_: &[u8]) -> Self {
        unimplemented!()
    }

    pub fn from_signature_keypair(_: &signature::KeyPair) -> Self {
        unimplemented!()
    }

    pub fn private_key(&self) -> &PrivateKey {
        &self.sk
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.pk
    }

    pub fn clear(&mut self) {
        self.sk.clear();
        self.pk.clear();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CryptoBox {
    key: [u8; Self::SYMMETRIC_KEY_BYTES]
}

impl CryptoBox {
    pub const SYMMETRIC_KEY_BYTES: usize = 32;
    pub const MAC_BYTES: usize = 16;

    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn from(_: &PublicKey, _: &PrivateKey) -> Self {
        unimplemented!()
    }

    pub fn size(&self) -> usize {
        Self::SYMMETRIC_KEY_BYTES
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.key.as_slice()
    }

    pub fn clear(&mut self) {
        self.key.fill(0)
    }

    pub fn encrypt(&self, _: &mut [u8], _: &[u8], _: &Nonce) {
        unimplemented!()
    }

    pub fn encrypt_into(&self, plain: &[u8], nonce: &Nonce) -> Vec<u8> {
        let mut cipher = vec!(0u8; plain.len() + Self::MAC_BYTES);
        self.encrypt(cipher.as_mut_slice(), plain, nonce);
        cipher
    }

    pub fn decrypt(&self, _: &mut[u8], _: &[u8], _: &Nonce) {
        unimplemented!()
    }

    pub fn decrypt_into(&self, cipher: &[u8], nonce: &Nonce) -> Vec<u8> {
        let mut plain = vec!(0u8; cipher.len() - Self::MAC_BYTES);
        self.decrypt(plain.as_mut_slice(), cipher, nonce);
        plain
    }
}

pub fn encrypt(_: &mut [u8], _: &[u8], _: &Nonce, _: &PublicKey, _: &PrivateKey) {
    unimplemented!()
}

pub fn encrypt_into(plain: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Vec<u8> {
    let mut cipher = vec!(0u8; plain.len() + CryptoBox::MAC_BYTES);
    encrypt(cipher.as_mut_slice(), plain, nonce, pk, sk);
    cipher
}

pub fn decrypt(_: &mut [u8], _: &[u8], _: &Nonce, _: &PublicKey, _: &PrivateKey) {
    unimplemented!()
}

pub fn decrypt_into(cipher: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Vec<u8> {
    let mut plain = vec!(0u8; cipher.len() - CryptoBox::MAC_BYTES);
    decrypt(plain.as_mut_slice(), cipher, nonce, pk, sk);
    plain
}
