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
use crate::{as_uchar_ptr, as_uchar_ptr_mut};
use crate::error::Error;

const_assert!(PrivateKey::BYTES == crypto_box_SECRETKEYBYTES as usize);
const_assert!(PublicKey::BYTES == crypto_box_PUBLICKEYBYTES as usize);
const_assert!(Nonce::BYTES == crypto_box_NONCEBYTES as usize);
const_assert!(KeyPair::SEED_BYTES == crypto_box_SEEDBYTES as usize);
const_assert!(CryptoBox::SYMMETRIC_KEY_BYTES == crypto_box_BEFORENMBYTES as usize);
const_assert!(CryptoBox::MAC_BYTES == crypto_box_MACBYTES as usize);

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct PrivateKey {
    key: [u8; Self::BYTES]
}

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

        PrivateKey {
            key: input.try_into().unwrap()
        }
    }

    pub fn from_signature_key(sk: &signature::PrivateKey) -> Result<Self, Error> {
        let mut input = [0u8; Self::BYTES];
        unsafe {
            let result = crypto_sign_ed25519_sk_to_curve25519(
                as_uchar_ptr_mut!(input),
                as_uchar_ptr!(sk.as_bytes())
            );

            if result != 0 {
                return Err(Error::Crypto(
                    format!("converts Ed25519 key to x25519 key failed.")
                ));
            }
        }
        Ok(PrivateKey {
            key: input
        })
    }

    pub const fn size(&self) -> usize {
        Self::BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.key.as_slice()
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct PublicKey {
    key: [u8; Self::BYTES]
}

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

        PublicKey {
            key: input.try_into().unwrap()
        }
    }

    pub fn from_signature_key(pk: &signature::PublicKey) -> Result<Self, Error> {
        let mut input = [0u8; Self::BYTES];
        unsafe { // Always success.
            let result = crypto_sign_ed25519_pk_to_curve25519(
                as_uchar_ptr_mut!(input),
                as_uchar_ptr!(pk.as_bytes())
            );

            if result != 0 {
                return Err(Error::Crypto(
                    format!("converts Ed25519 key to x25519 key failed.")
                ));
            }
        }
        Ok(PublicKey {
            key: input
        })
    }

    pub const fn size(&self) -> usize {
        Self::BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.key.as_slice()
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Nonce {
    nonce: [u8; Self::BYTES]
}

impl Nonce {
    pub const BYTES: usize = 24;

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
                as_uchar_ptr_mut!(self.nonce),
                Self::BYTES
            )
        }
        self
    }

    pub const fn size(&self) -> usize {
        Self::BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
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

#[derive(Clone, Debug)]
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
                as_uchar_ptr_mut!(pk),
                as_uchar_ptr_mut!(sk)
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
                as_uchar_ptr_mut!(pk),
                as_uchar_ptr!(sk)
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
                as_uchar_ptr_mut!(pk),
                as_uchar_ptr!(sk.as_bytes())
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
                as_uchar_ptr_mut!(pk),
                as_uchar_ptr_mut!(sk),
                as_uchar_ptr!(seed)
            );
        }

        KeyPair {
            sk: PrivateKey::from(sk.as_slice()),
            pk: PublicKey::from(pk.as_slice())
        }
    }

    pub fn from_signature_keypair(keypair: &signature::KeyPair) -> Self {
        let mut x25519 = vec!(0u8; crypto_box_SECRETKEYBYTES as usize);

        unsafe {
            crypto_sign_ed25519_sk_to_curve25519(
                as_uchar_ptr_mut!(x25519),
                as_uchar_ptr!(keypair.private_key().as_bytes())
            );
        }

        Self::from(x25519.as_slice())
    }

    pub const fn private_key(&self) -> &PrivateKey {
        &self.sk
    }

    pub const fn public_key(&self) -> &PublicKey {
        &self.pk
    }

    pub fn clear(&mut self) {
        self.sk.clear();
        self.pk.clear();
    }
}

#[derive(Default, Debug)]
pub struct CryptoBox {
    key: [u8; Self::SYMMETRIC_KEY_BYTES]
}

impl CryptoBox {
    pub const SYMMETRIC_KEY_BYTES: usize = 32;
    pub const MAC_BYTES: usize = 16;

    pub fn try_from(pk: &PublicKey, sk: &PrivateKey) -> Result<Self, Error> {
        let mut k = vec!(0u8; Self::SYMMETRIC_KEY_BYTES);
        unsafe {
            let result = crypto_box_beforenm(
                as_uchar_ptr_mut!(k),
                as_uchar_ptr!(pk.as_bytes()),
                as_uchar_ptr!(sk.as_bytes())
            );

            if result != 0 {
                return Err(Error::Crypto(
                    format!("Compute symmetric key failed, wrong pk or sk")
                ));
            }
        }
        Ok(CryptoBox {
            key: k.try_into().unwrap()
        })
    }

    pub const fn size(&self) -> usize {
        Self::SYMMETRIC_KEY_BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
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
                as_uchar_ptr_mut!(cipher),
                as_uchar_ptr!(plain),
                plain.len() as libc::c_ulonglong,
                as_uchar_ptr!(nonce.as_bytes()),
                as_uchar_ptr!(self.key)
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
                as_uchar_ptr_mut!(plain),
                as_uchar_ptr!(cipher),
                cipher.len() as libc::c_ulonglong,
                as_uchar_ptr!(nonce.as_bytes()),
                as_uchar_ptr!(self.key)
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

pub fn encrypt(cipher: &mut [u8], plain: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Result<(), Error> {
    if cipher.len() < plain.len() + crypto_box_MACBYTES as usize {
        return Err(Error::Argument(format!("The cipher buffer is too small")));
    }

    unsafe {
        let result = crypto_box_easy(
            as_uchar_ptr_mut!(cipher),
            as_uchar_ptr!(plain),
            plain.len() as libc::c_ulonglong,
            as_uchar_ptr!(nonce.as_bytes()),
            as_uchar_ptr!(pk.as_bytes()),
            as_uchar_ptr!(sk.as_bytes())
        );
        if result != 0 {
            return Err(Error::Crypto(format!("Encrypt data failed")));
        }
    }
    Ok(())
}

pub fn encrypt_into(plain: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Vec<u8> {
    let mut cipher = vec!(0u8; plain.len() + CryptoBox::MAC_BYTES);
    encrypt(cipher.as_mut_slice(), plain, nonce, pk, sk).unwrap();
    cipher
}

pub fn decrypt(plain: &mut [u8], cipher: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Result<(), Error> {
    if plain.len() < cipher.len() - crypto_box_MACBYTES as usize {
        return Err(Error::Argument(format!("The plain buffer is too small")));
    }

    unsafe {
        let result = crypto_box_open_easy(
            as_uchar_ptr_mut!(plain),
            as_uchar_ptr!(cipher),
            cipher.len() as libc::c_ulonglong,
            as_uchar_ptr!(nonce.as_bytes()),
            as_uchar_ptr!(pk.as_bytes()),
            as_uchar_ptr!(sk.as_bytes())
        );

        if result != 0 {
            return Err(Error::Crypto(format!("Decrypt data failed")))
        }
    }
    Ok(())
}

pub fn decrypt_into(cipher: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &PrivateKey) -> Vec<u8> {
    let mut plain = vec!(0u8; cipher.len() - CryptoBox::MAC_BYTES);
    decrypt(plain.as_mut_slice(), cipher, nonce, pk, sk).unwrap();
    plain
}
