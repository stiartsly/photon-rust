use static_assertions::const_assert;
use libsodium_sys::{
    crypto_sign_detached,
    crypto_sign_verify_detached,
    crypto_sign_keypair,
    crypto_sign_SECRETKEYBYTES,
    crypto_sign_PUBLICKEYBYTES,
    crypto_sign_SEEDBYTES,
    crypto_sign_BYTES,
};

const_assert!(PrivateKey::BYTES == crypto_sign_SECRETKEYBYTES as usize);
const_assert!(PublicKey::BYTES == crypto_sign_PUBLICKEYBYTES as usize);
const_assert!(KeyPair::SEED_BYTES == crypto_sign_SEEDBYTES as usize);
const_assert!(Signature::BYTES == crypto_sign_BYTES as usize);

#[derive(Debug, Clone, Copy)]
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
            );
            Ok(true)
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct KeyPair {
    sk: PrivateKey,
    pk: PublicKey
}

impl KeyPair {
    pub const SEED_BYTES: usize = 32;

    pub fn new() -> Self {
        let mut skey = vec![0u8; PrivateKey::BYTES];
        let mut pkey = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_sign_keypair(
                pkey.as_mut_ptr(),
                skey.as_mut_ptr()
            );
        }
        KeyPair {
            sk: PrivateKey::from(&skey).unwrap(),
            pk: PublicKey::from(&pkey).unwrap()
        }
    }

    pub fn with_private_key(sk: &PrivateKey) -> Self {
        KeyPair {
            sk: *sk,
            pk: PublicKey::new()
        }
    }

    pub fn with_seed(_: &Vec<u8>) -> Self {
        KeyPair {
            sk: PrivateKey::new(),
            pk: PublicKey::new()
        }
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
}
