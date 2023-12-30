use std::fmt;
use static_assertions::const_assert;
use libsodium_sys::{
    randombytes_buf,
    crypto_sign_detached,
    crypto_sign_verify_detached,
    crypto_sign_keypair,
    crypto_sign_ed25519_sk_to_pk,
    crypto_sign_seed_keypair,
    crypto_sign_SECRETKEYBYTES,
    crypto_sign_PUBLICKEYBYTES,
    crypto_sign_SEEDBYTES,
    crypto_sign_BYTES,
};

const_assert!(PrivateKey::BYTES == crypto_sign_SECRETKEYBYTES as usize);
const_assert!(PublicKey::BYTES == crypto_sign_PUBLICKEYBYTES as usize);
const_assert!(KeyPair::SEED_BYTES == crypto_sign_SEEDBYTES as usize);
const_assert!(Signature::BYTES == crypto_sign_BYTES as usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrivateKey {
    key: [u8; Self::BYTES]
}

impl PrivateKey {
    pub const BYTES: usize = 64;

    pub fn new() -> Self {
        PrivateKey {
            key: [0; Self::BYTES]
        }
    }

    pub fn from<'a>(key: &'a [u8]) -> Result<Self, &'static str> {
        if key.len() != Self::BYTES {
            return Err("Incorrect raw private key size");
        }

        let sk: [u8; Self::BYTES] = key.try_into().map_err(|_| "Conversion slice failed")?;
        Ok(PrivateKey { key: sk })
    }

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn bytes(&self) -> &[u8; Self::BYTES] {
        &self.key
    }

    pub fn clear(&mut self) {
        self.key.fill(0);
    }

    pub fn sign<'a>(&self, data: &'a [u8], signature: &'a mut[u8]) -> Result<bool, &'static str> {
        if signature.len() != Signature::BYTES {
            return Err("Invalid signature length");
        }

        unsafe {
            crypto_sign_detached(
                signature.as_mut_ptr(),
                std::ptr::null_mut(),
                data.as_ptr(),
                data.len() as u64,
                self.key.as_ptr()
            ); // Always success
        }
        Ok(true)
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = hex::encode(self.key);
        write!(f, "{}", str)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicKey {
    key: [u8; Self::BYTES]
}

impl PublicKey {
    pub const BYTES: usize = 32;

    pub fn new() -> Self {
        PublicKey {
            key: [0; Self::BYTES]
        }
    }

    pub fn from<'a>(key: &'a [u8]) -> Result<Self, &'static str> {
        if key.len() != Self::BYTES {
            return Err("Incorrect raw private key size");
        }

        let pk: [u8; Self::BYTES] = key.try_into().map_err(|_| "Conversion slice failed")?;
        Ok(PublicKey { key: pk })
    }

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn bytes(&self) -> &[u8; Self::BYTES] {
        &self.key
    }

    pub fn clear(&mut self) {
        self.key.fill(0);
    }

    pub fn verify<'a>(&self, data: &'a [u8], signature: &'a [u8]) -> Result<bool, &'static str> {
        if signature.len() != Signature::BYTES {
            return Err("Invalid signature length");
        }

        unsafe {
            let rc = crypto_sign_verify_detached(
                signature.as_ptr(),
                data.as_ptr(),
                data.len() as u64,
                self.key.as_ptr(),
            );

            match rc {
                0 => Ok(true),
                _ => Err("Verification failed")
            }
        }
    }

}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = hex::encode(self.key);
        write!(f, "{}", str)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyPair {
    sk: PrivateKey,
    pk: PublicKey
}

impl KeyPair {
    pub const SEED_BYTES: usize = 32;

    pub fn new() -> Self {
        let mut sk = vec![0u8; PrivateKey::BYTES];
        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_sign_keypair(
                pk.as_mut_ptr(),
                sk.as_mut_ptr()
            ); // Always success
        }
        KeyPair {
            sk: PrivateKey::from(&sk).unwrap(),
            pk: PublicKey::from(&pk).unwrap(),
        }
    }

    pub fn random() -> Self {
        let mut seed = [0u8; KeyPair::SEED_BYTES];
        unsafe {
            randombytes_buf(
                seed.as_mut_ptr() as *mut libc::c_void,
                KeyPair::SEED_BYTES
            ); // Always success.
        }

        let mut sk = vec![0u8; PrivateKey::BYTES];
        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_sign_seed_keypair(
                pk.as_mut_ptr(),
                sk.as_mut_ptr(),
                seed.as_ptr()
            ); // Always success
        }
        KeyPair {
            sk: PrivateKey::from(&sk).unwrap(),
            pk: PublicKey::from(&pk).unwrap()
        }
    }

    pub fn from_private_key(private_key: &PrivateKey) -> Self {
        let sk = private_key.clone();
        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_sign_ed25519_sk_to_pk(
                pk.as_mut_ptr(),
                private_key.bytes().as_ptr()
            ); // Always success
        }
        KeyPair {
            sk,
            pk: PublicKey::from(&pk).unwrap()
        }
    }

    pub fn from_private_key_data(key: &[u8]) -> Result<Self, &'static str> {
        if key.len() != PrivateKey::BYTES {
            return Err("Incorrect private key size");
        }

        let mut pk = vec![0u8; PublicKey::BYTES];
        unsafe {
            crypto_sign_ed25519_sk_to_pk(
                pk.as_mut_ptr(),
                key.as_ptr()
            ); // Always success
        }

        Ok(KeyPair {
            sk: PrivateKey::from(key)?,
            pk: PublicKey::from(&pk)?
        })
    }

    pub fn from_seed<'a>(seed: &'a [u8]) -> Result<Self, &'static str> {
        if seed.len() != KeyPair::SEED_BYTES {
            return Err("Incorrect seed size");
        }

        let mut sk = vec![0u8; PrivateKey::BYTES];
        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_sign_seed_keypair(
                pk.as_mut_ptr(),
                sk.as_mut_ptr(),
                seed.as_ptr()
            ); // Always success
        }
        Ok(
            KeyPair {
                sk: PrivateKey::from(&sk)?,
                pk: PublicKey::from(&pk)?
            }
        )
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

#[derive(Debug)]
pub struct Signature {
    // Define the fields of Signature as needed
}

impl Signature {
    pub const BYTES: usize = 64;

    pub fn verify<'a>(_: &[u8], _: &[u8], _: &PublicKey) -> Result<bool, &'static str> {
        Err("TODO")
    }
}
