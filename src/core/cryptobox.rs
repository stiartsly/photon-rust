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
    crypto_sign_ed25519_pk_to_curve25519,
    randombytes_buf,
    sodium_increment,
    crypto_box_keypair,
    crypto_scalarmult_base,
    crypto_box_seed_keypair,
    crypto_box_beforenm,
    crypto_box_easy_afternm,
    crypto_box_open_easy_afternm,
    crypto_box_easy,
    crypto_box_open_easy
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
        PrivateKey { key: [0; Self::BYTES] }
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
        let mut nonce = [0u8; Self::BYTES];
        unsafe { // Always success.
            randombytes_buf(
                nonce.as_mut_ptr() as *mut libc::c_void,
                Self::BYTES
            );
        }
        Nonce { nonce }
    }

    pub fn increment(&mut self) -> &Self {
        unsafe { // Always success.
            sodium_increment(
                self.nonce.as_mut_ptr() as *mut libc::c_uchar,
                Self::BYTES
            )
        }
        self
    }

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.nonce.as_slice()
    }

    pub fn clear(&mut self) {
        self.nonce.fill(0)
    }
}

impl std::fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.nonce))?;
        Ok(())
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
        let mut pk = vec!(0u8; PublicKey::BYTES);
        let mut sk = vec!(0u8; PrivateKey::BYTES);

        unsafe {// Always success.
            crypto_box_keypair(
                pk.as_mut_ptr() as *mut libc::c_uchar,
                sk.as_mut_ptr() as *mut libc::c_uchar
            );
        }

        KeyPair {
            sk: PrivateKey::from(sk.as_slice()),
            pk: PublicKey::from(pk.as_slice())
        }
    }

    pub fn from(sk: &[u8]) -> Self {
        assert_eq!(
            sk.len(),
            PrivateKey::BYTES,
            "Invalid raw private key size {}, should be {}",
            sk.len(),
            PrivateKey::BYTES
        );

        let mut pk = vec!(0u8; PublicKey::BYTES);

        unsafe {
            crypto_scalarmult_base(
                pk.as_mut_ptr() as *mut libc::c_uchar,
                sk.as_ptr() as *const libc::c_uchar
            );
        }

        KeyPair {
            sk: PrivateKey::from(sk),
            pk: PublicKey::from(pk.as_slice())
        }
    }

    pub fn from_private_key(sk: &PrivateKey) -> Self {
        let mut pk = vec!(0u8; PublicKey::BYTES);

        unsafe {
            crypto_scalarmult_base(
                pk.as_mut_ptr() as *mut libc::c_uchar,
                sk.as_bytes().as_ptr() as *const libc::c_uchar,
            );
        }

        KeyPair {
            sk: sk.clone(),
            pk: PublicKey::from(pk.as_slice())
        }
    }

    pub fn from_seed(seed: &[u8]) -> Self {
        assert_eq!(
            seed.len(),
            KeyPair::SEED_BYTES,
            "Invalid seed size {}, should be {}",
            seed.len(),
            KeyPair::SEED_BYTES
        );

        let mut pk = vec!(0u8; PublicKey::BYTES);
        let mut sk = vec!(0u8; PrivateKey::BYTES);

        unsafe {
            crypto_box_seed_keypair(
                pk.as_mut_ptr() as *mut libc::c_uchar,
                sk.as_mut_ptr() as *mut libc::c_uchar,
                seed.as_ptr() as *const libc::c_uchar
            );
        }

        KeyPair {
            sk: PrivateKey::from(sk.as_slice()),
            pk: PublicKey::from(pk.as_slice())
        }
    }

    pub fn from_signature_keypair(sign_keypair: &signature::KeyPair) -> Self {
        let mut x25519 = vec!(0u8; crypto_box_SECRETKEYBYTES as usize);

        unsafe {
            crypto_sign_ed25519_sk_to_curve25519(
                x25519.as_mut_ptr() as *mut libc::c_uchar,
                sign_keypair.private_key().as_bytes().as_ptr() as *const libc::c_uchar
            );
        }

        Self::from(x25519.as_slice())
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
        CryptoBox {
            key: [0u8; Self::SYMMETRIC_KEY_BYTES]
        }
    }

    pub fn try_from(pk: &PublicKey, sk: &PrivateKey) -> Result<Self, Error> {
        let mut k = vec!(0u8; Self::SYMMETRIC_KEY_BYTES);
        unsafe {
            let result = crypto_box_beforenm(
                k.as_mut_ptr() as *mut libc::c_uchar,
                pk.as_bytes().as_ptr() as *const libc::c_uchar,
                sk.as_bytes().as_ptr() as *const libc::c_uchar
            );

            if result != 0 {
                return Err(Error::Crypto(format!("Compute symmetric key failed, wrong pk or sk")));
            }
        }

        Ok(CryptoBox {
            key: k.try_into().unwrap()
        })
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

    pub fn encrypt(&self, cipher: &mut [u8], plain: &[u8], nonce: &Nonce) -> Result<(), Error>{
        if cipher.len() < plain.len() + crypto_box_MACBYTES as usize {
            return Err(Error::Argument(format!("The cipher buffer is too small")));
        }

        unsafe {
            let result = crypto_box_easy_afternm(
                cipher.as_mut_ptr() as *mut libc::c_uchar,
                plain.as_ptr() as *const libc::c_uchar,
                plain.len() as libc::c_ulonglong,
                nonce.as_bytes().as_ptr() as *const libc::c_uchar,
                self.key.as_ptr() as *const libc::c_uchar
            );
            if result != 0 {
                return Err(Error::Crypto(format!("Encrypt data failed")));
            }
        }

        Ok(())
    }

    pub fn encrypt_into(&self, plain: &[u8], nonce: &Nonce) -> Vec<u8> {
        let mut cipher = vec!(0u8; plain.len() + Self::MAC_BYTES);
        self.encrypt(cipher.as_mut_slice(), plain, nonce).unwrap();
        cipher
    }

    pub fn decrypt(&self, plain: &mut[u8], cipher: &[u8], nonce: &Nonce) -> Result<(), Error> {
        if plain.len() < cipher.len() - crypto_box_MACBYTES as usize {
            return Err(Error::Argument(format!("The plain buffer is too small")));
        }

        unsafe {
            let result = crypto_box_open_easy_afternm(
                plain.as_mut_ptr() as *mut libc::c_uchar,
                cipher.as_ptr() as *const libc::c_uchar,
                cipher.len() as libc::c_ulonglong,
                nonce.as_bytes().as_ptr() as *const libc::c_uchar,
                self.key.as_ptr() as *const libc::c_uchar
            );
            if result != 0 {
                return Err(Error::Crypto(format!("Encrypt data failed")));
            }
        }

        Ok(())
    }

    pub fn decrypt_into(&self, cipher: &[u8], nonce: &Nonce) -> Vec<u8> {
        let mut plain = vec!(0u8; cipher.len() - Self::MAC_BYTES);
        self.decrypt(plain.as_mut_slice(), cipher, nonce).unwrap();
        plain
    }
}

pub fn encrypt(cipher: &mut [u8],
    plain: &[u8],
    nonce: &Nonce,
    pk: &PublicKey,
    sk: &PrivateKey
) -> Result<(), Error> {

    if cipher.len() < plain.len() + crypto_box_MACBYTES as usize {
        return Err(Error::Argument(format!("The cipher buffer is too small")));
    }

    unsafe {
        let result = crypto_box_easy(
            cipher.as_mut_ptr() as *mut libc::c_uchar,
            plain.as_ptr() as *const libc::c_uchar,
            plain.len() as libc::c_ulonglong,
            nonce.as_bytes().as_ptr() as *const libc::c_uchar,
            pk.as_bytes().as_ptr() as *const libc::c_uchar,
            sk.as_bytes().as_ptr() as *const libc::c_uchar
        );
        if result != 0 {
            return Err(Error::Crypto(format!("Encrypt data failed")));
        }
    }
    Ok(())
}

pub fn encrypt_into(plain: &[u8],
    nonce: &Nonce,
    pk: &PublicKey,
    sk: &PrivateKey
) -> Vec<u8> {
    let mut cipher = vec!(0u8; plain.len() + CryptoBox::MAC_BYTES);
    encrypt(cipher.as_mut_slice(), plain, nonce, pk, sk).unwrap();
    cipher
}

/*
void CryptoBox::decrypt(Blob& plain, const Blob& cipher,
        const PublicKey& pk, const PrivateKey& sk, const Nonce& nonce)
{
    assert(plain.size() >= cipher.size() - crypto_box_MACBYTES);
    if (plain.size() < cipher.size() - crypto_box_MACBYTES)
        throw std::invalid_argument("The plain buffer is too small.");

    if (crypto_box_open_easy(plain.ptr(), cipher.ptr(), cipher.size(), nonce.bytes(), pk.bytes(), sk.bytes()) != 0)
        throw CryptoError(std::string("Decrypt data failed."));
}*/

pub fn decrypt(plain: &mut [u8],
    cipher: &[u8],
    nonce: &Nonce,
    pk: &PublicKey,
    sk: &PrivateKey
) -> Result<(), Error> {
    if plain.len() < cipher.len() - crypto_box_MACBYTES as usize {
        return Err(Error::Argument(format!("The plain buffer is too small")));
    }

    unsafe {
        let result = crypto_box_open_easy(
            plain.as_mut_ptr() as *mut libc::c_uchar,
            cipher.as_ptr() as *const libc::c_uchar,
            cipher.len() as libc::c_ulonglong,
            nonce.as_bytes().as_ptr() as *const libc::c_uchar,
            pk.as_bytes().as_ptr() as *const libc::c_uchar,
            sk.as_bytes().as_ptr() as *const libc::c_uchar
        );

        if result != 0 {
            return Err(Error::Crypto(format!("Decrypt data failed")))
        }
    }

    Ok(())
}

pub fn decrypt_into(cipher: &[u8],
    nonce: &Nonce,
    pk: &PublicKey,
    sk: &PrivateKey
) -> Vec<u8> {
    let mut plain = vec!(0u8; cipher.len() - CryptoBox::MAC_BYTES);
    decrypt(plain.as_mut_slice(), cipher, nonce, pk, sk).unwrap();
    plain
}
