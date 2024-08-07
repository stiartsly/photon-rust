
use std::fmt;
use static_assertions::const_assert;
use libsodium_sys::{
    crypto_box_BEFORENMBYTES,
    crypto_box_MACBYTES,
    crypto_box_NONCEBYTES,
    crypto_box_PUBLICKEYBYTES,
    crypto_box_SECRETKEYBYTES,
    crypto_box_SEEDBYTES,
    crypto_box_beforenm,
    crypto_box_easy,
    crypto_box_easy_afternm,
    crypto_box_keypair,
    crypto_box_open_easy,
    crypto_box_open_easy_afternm,
    crypto_box_seed_keypair,
    crypto_scalarmult_base,
    crypto_sign_ed25519_pk_to_curve25519,
    crypto_sign_ed25519_sk_to_curve25519,
    randombytes_buf,
    sodium_increment,
};

use crate::{
    as_uchar_ptr,
    as_uchar_ptr_mut,
    signature,
    error::Error
};

const_assert!(PrivateKey::BYTES == crypto_box_SECRETKEYBYTES as usize);
const_assert!(PublicKey::BYTES == crypto_box_PUBLICKEYBYTES as usize);
const_assert!(Nonce::BYTES == crypto_box_NONCEBYTES as usize);
const_assert!(KeyPair::SEED_BYTES == crypto_box_SEEDBYTES as usize);
const_assert!(CryptoBox::SYMMETRIC_KEY_BYTES == crypto_box_BEFORENMBYTES as usize);
const_assert!(CryptoBox::MAC_BYTES == crypto_box_MACBYTES as usize);

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct PrivateKey([u8; Self::BYTES]);

impl PrivateKey {
    pub const BYTES: usize = 32;

    pub fn from(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            Self::BYTES,
            "Incorrect raw private key size {}, should be {}",
            input.len(),
            Self::BYTES
        );

        PrivateKey(input.try_into().unwrap())
    }

    pub fn from_signature_key(sk: &signature::PrivateKey) -> Result<Self, Error> {
        let mut input = [0u8; Self::BYTES];
        unsafe {
            let rc = crypto_sign_ed25519_sk_to_curve25519(
                as_uchar_ptr_mut!(input),
                as_uchar_ptr!(sk.as_bytes()),
            );

            if rc != 0 {
                return Err(Error::Crypto(format!(
                    "converts Ed25519 key to x25519 key failed."
                )));
            }
        }
        Ok(PrivateKey(input))
    }

    pub const fn size(&self) -> usize {
        Self::BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn clear(&mut self) {
        self.0.fill(0);
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))?;
        Ok(())
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct PublicKey([u8; Self::BYTES]);

impl PublicKey {
    pub const BYTES: usize = 32;

    pub fn from(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            Self::BYTES,
            "Incorrect raw public key size {}, should be {}",
            input.len(),
            Self::BYTES
        );

        PublicKey(input.try_into().unwrap())
    }

    pub fn from_signature_key(pk: &signature::PublicKey) -> Result<Self, Error> {
        let mut input = [0u8; Self::BYTES];
        unsafe {
            // Always success.
            let rc = crypto_sign_ed25519_pk_to_curve25519(
                as_uchar_ptr_mut!(input),
                as_uchar_ptr!(pk.as_bytes()),
            );

            if rc != 0 {
                return Err(Error::Crypto(format!(
                    "converts Ed25519 key to x25519 key failed."
                )));
            }
        }
        Ok(PublicKey(input))
    }

    pub const fn size(&self) -> usize {
        Self::BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn clear(&mut self) {
        self.0.fill(0);
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))?;
        Ok(())
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Nonce([u8; Self::BYTES]);

impl Nonce {
    pub const BYTES: usize = 24;

    pub fn random() -> Self {
        let mut bytes = [0u8; Self::BYTES];
        unsafe {
            // Always success.
            randombytes_buf(bytes.as_mut_ptr() as *mut libc::c_void, Self::BYTES);
        }
        Nonce(bytes)
    }

    pub fn from(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            Self::BYTES,
            "Incorrect nonce size {}, should be {}",
            input.len(),
            Self::BYTES
        );

        Nonce(input.try_into().unwrap())
    }

    pub fn increment(&mut self) -> &Self {
        unsafe {
            // Always success.
            sodium_increment(as_uchar_ptr_mut!(self.0), Self::BYTES)
        }
        self
    }

    pub const fn size(&self) -> usize {
        Self::BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn clear(&mut self) {
        self.0.fill(0)
    }
}

impl std::fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct KeyPair(PrivateKey, PublicKey);

impl KeyPair {
    pub const SEED_BYTES: usize = 32;

    pub fn new() -> Self {
        let mut pk = vec![0u8; PublicKey::BYTES];
        let mut sk = vec![0u8; PrivateKey::BYTES];

        unsafe {
            // Always success.
            crypto_box_keypair(as_uchar_ptr_mut!(pk), as_uchar_ptr_mut!(sk));
        }

        KeyPair(
            PrivateKey::from(sk.as_slice()),
            PublicKey::from(pk.as_slice()),
        )
    }

    pub fn from(sk: &[u8]) -> Self {
        assert_eq!(
            sk.len(),
            PrivateKey::BYTES,
            "Invalid raw private key size {}, should be {}",
            sk.len(),
            PrivateKey::BYTES
        );

        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_scalarmult_base(as_uchar_ptr_mut!(pk), as_uchar_ptr!(sk));
        }

        KeyPair(PrivateKey::from(sk), PublicKey::from(pk.as_slice()))
    }

    pub fn from_private_key(sk: &PrivateKey) -> Self {
        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_scalarmult_base(as_uchar_ptr_mut!(pk), as_uchar_ptr!(sk.as_bytes()));
        }

        KeyPair(sk.clone(), PublicKey::from(pk.as_slice()))
    }

    pub fn from_seed(seed: &[u8]) -> Self {
        assert_eq!(
            seed.len(),
            KeyPair::SEED_BYTES,
            "Invalid seed size {}, should be {}",
            seed.len(),
            KeyPair::SEED_BYTES
        );

        let mut pk = vec![0u8; PublicKey::BYTES];
        let mut sk = vec![0u8; PrivateKey::BYTES];

        unsafe {
            crypto_box_seed_keypair(
                as_uchar_ptr_mut!(pk),
                as_uchar_ptr_mut!(sk),
                as_uchar_ptr!(seed),
            );
        }

        KeyPair(
            PrivateKey::from(sk.as_slice()),
            PublicKey::from(pk.as_slice()),
        )
    }

    pub fn from_signature_keypair(keypair: &signature::KeyPair) -> Self {
        let mut x25519 = vec![0u8; crypto_box_SECRETKEYBYTES as usize];

        unsafe {
            crypto_sign_ed25519_sk_to_curve25519(
                as_uchar_ptr_mut!(x25519),
                as_uchar_ptr!(keypair.private_key().as_bytes()),
            );
        }

        Self::from(x25519.as_slice())
    }

    pub const fn private_key(&self) -> &PrivateKey {
        &self.0
    }

    pub const fn public_key(&self) -> &PublicKey {
        &self.1
    }

    pub fn clear(&mut self) {
        self.0.clear();
        self.1.clear();
    }
}

#[derive(Default, Debug)]
pub struct CryptoBox([u8; Self::SYMMETRIC_KEY_BYTES]);

impl CryptoBox {
    pub const SYMMETRIC_KEY_BYTES: usize = 32;
    pub const MAC_BYTES: usize = 16;

    pub fn try_from(pk: &PublicKey, sk: &PrivateKey) -> Result<Self, Error> {
        let mut k = vec![0u8; Self::SYMMETRIC_KEY_BYTES];
        unsafe {
            let rc = crypto_box_beforenm(
                as_uchar_ptr_mut!(k),
                as_uchar_ptr!(pk.as_bytes()),
                as_uchar_ptr!(sk.as_bytes()),
            );

            if rc != 0 {
                return Err(Error::Crypto(format!(
                    "Compute symmetric key failed, wrong pk or sk"
                )));
            }
        }
        Ok(CryptoBox(k.try_into().unwrap()))
    }

    pub const fn size(&self) -> usize {
        Self::SYMMETRIC_KEY_BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn clear(&mut self) {
        self.0.fill(0)
    }

    pub fn encrypt(&self, plain: &[u8], cipher: &mut [u8], nonce: &Nonce) -> Result<(), Error> {
        if cipher.len() < plain.len() + crypto_box_MACBYTES as usize {
            return Err(Error::Argument(format!("The cipher buffer is too small")));
        }

        unsafe {
            let rc = crypto_box_easy_afternm(
                as_uchar_ptr_mut!(cipher),
                as_uchar_ptr!(plain),
                plain.len() as libc::c_ulonglong,
                as_uchar_ptr!(nonce.as_bytes()),
                as_uchar_ptr!(self.0),
            );
            if rc != 0 {
                return Err(Error::Crypto(format!("Encrypt data failed")));
            }
        }
        Ok(())
    }

    pub fn encrypt_into(&self, plain: &[u8], nonce: &Nonce) -> Result<Vec<u8>, Error> {
        let mut cipher = vec![0u8; plain.len() + Self::MAC_BYTES];
        self.encrypt(plain, cipher.as_mut_slice(), nonce).map(|_| cipher)
    }

    pub fn decrypt(&self, cipher: &[u8], plain: &mut [u8], nonce: &Nonce) -> Result<(), Error> {
        if plain.len() < cipher.len() - crypto_box_MACBYTES as usize {
            return Err(Error::Argument(format!("The plain buffer is too small")));
        }

        unsafe {
            let rc = crypto_box_open_easy_afternm(
                as_uchar_ptr_mut!(plain),
                as_uchar_ptr!(cipher),
                cipher.len() as libc::c_ulonglong,
                as_uchar_ptr!(nonce.as_bytes()),
                as_uchar_ptr!(self.0),
            );
            if rc != 0 {
                return Err(Error::Crypto(format!("Encrypt data failed")));
            }
        }
        Ok(())
    }

    pub fn decrypt_into(&self, cipher: &[u8], nonce: &Nonce) -> Result<Vec<u8>, Error> {
        let mut plain = vec![0u8; cipher.len() - Self::MAC_BYTES];
        self.decrypt(cipher, plain.as_mut_slice(), nonce).map(|_| plain)
    }
}

pub fn encrypt(
    cipher: &mut [u8],
    plain: &[u8],
    nonce: &Nonce,
    pk: &PublicKey,
    sk: &PrivateKey,
) -> Result<(), Error> {
    if cipher.len() < plain.len() + crypto_box_MACBYTES as usize {
        return Err(Error::Argument(format!("The cipher buffer is too small")));
    }

    unsafe {
        let rc = crypto_box_easy(
            as_uchar_ptr_mut!(cipher),
            as_uchar_ptr!(plain),
            plain.len() as libc::c_ulonglong,
            as_uchar_ptr!(nonce.as_bytes()),
            as_uchar_ptr!(pk.as_bytes()),
            as_uchar_ptr!(sk.as_bytes()),
        );
        if rc != 0 {
            return Err(Error::Crypto(format!("Encrypt data failed")));
        }
    }
    Ok(())
}

pub fn encrypt_into(plain: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Result<Vec<u8>, Error> {
    let mut cipher = vec![0u8; plain.len() + CryptoBox::MAC_BYTES];
    encrypt(cipher.as_mut_slice(), plain, nonce, pk, sk).map(|_| cipher)
}

pub fn decrypt(
    plain: &mut [u8],
    cipher: &[u8],
    nonce: &Nonce,
    pk: &PublicKey,
    sk: &PrivateKey,
) -> Result<(), Error> {
    if plain.len() < cipher.len() - crypto_box_MACBYTES as usize {
        return Err(Error::Argument(format!("The plain buffer is too small")));
    }

    unsafe {
        let rc = crypto_box_open_easy(
            as_uchar_ptr_mut!(plain),
            as_uchar_ptr!(cipher),
            cipher.len() as libc::c_ulonglong,
            as_uchar_ptr!(nonce.as_bytes()),
            as_uchar_ptr!(pk.as_bytes()),
            as_uchar_ptr!(sk.as_bytes()),
        );

        if rc != 0 {
            return Err(Error::Crypto(format!("Decrypt data failed")));
        }
    }
    Ok(())
}

pub fn decrypt_into(cipher: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Result<Vec<u8>, Error> {
    let mut plain = vec![0u8; cipher.len() - CryptoBox::MAC_BYTES];
    decrypt(plain.as_mut_slice(), cipher, nonce, pk, sk).map(|_| plain)
}
