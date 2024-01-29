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
    crypto_sign_state,
    crypto_sign_init,
    crypto_sign_update,
    crypto_sign_final_create,
    crypto_sign_final_verify
};

const_assert!(PrivateKey::BYTES == crypto_sign_SECRETKEYBYTES as usize);
const_assert!(PublicKey::BYTES == crypto_sign_PUBLICKEYBYTES as usize);
const_assert!(KeyPair::SEED_BYTES == crypto_sign_SEEDBYTES as usize);
const_assert!(Signature::BYTES == crypto_sign_BYTES as usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.key.as_ref()
    }

    pub fn clear(&mut self) {
        self.key.fill(0);
    }

    pub fn sign(&self, data: &[u8], signature: &mut[u8]) {
        assert_eq!(
            signature.len(),
            Signature::BYTES,
            "Incorrect signature length {}, should be {}",
            signature.len(),
            Signature::BYTES
        );

        unsafe {
            crypto_sign_detached(
                signature.as_mut_ptr(),
                std::ptr::null_mut(),
                data.as_ptr(),
                data.len() as u64,
                self.key.as_ptr()
            ); // Always success
        }
    }

    pub fn sign_into(&self, data: &[u8]) -> Vec<u8> {
        let mut signature = vec![0u8; Signature::BYTES];
        self.sign(data, signature.as_mut_slice());
        signature
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = hex::encode(self.key);
        write!(f, "{}", str)?;
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
        PublicKey {
            key: [0; Self::BYTES]
        }
    }

    pub fn from(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            Self::BYTES,
            "Incorrect raw public key size {}, should be {}",
            input.len(),
            Self::BYTES
        );

        let pk: [u8; Self::BYTES] = input.try_into().unwrap();
        PublicKey { key: pk }
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

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        assert_eq!(
            signature.len(),
            Signature::BYTES,
            "Incorrect signature length {}, should be {}",
            signature.len(),
            Signature::BYTES
        );

        unsafe {
            let rc = crypto_sign_verify_detached(
                signature.as_ptr(),
                data.as_ptr(),
                data.len() as u64,
                self.key.as_ptr(),
            );

            match rc {
                0 => true,
                _ => false
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

        unsafe { // Always success
            crypto_sign_keypair(
                pk.as_mut_ptr(),
                sk.as_mut_ptr()
            );
        }
        KeyPair {
            sk: PrivateKey::from(&sk),
            pk: PublicKey::from(&pk),
        }
    }

    pub fn from_private_key(input: &PrivateKey) -> Self {
        let sk = input.clone();
        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_sign_ed25519_sk_to_pk(
                pk.as_mut_ptr(),
                input.as_bytes().as_ptr()
            ); // Always success
        }
        KeyPair {
            sk,
            pk: PublicKey::from(&pk)
        }
    }

    pub fn from_private_key_bytes(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            PrivateKey::BYTES,
            "Incorrect private key size {}, should be {}",
            input.len(),
            PrivateKey::BYTES
        );

        let mut pk = vec![0u8; PublicKey::BYTES];
        unsafe {
            crypto_sign_ed25519_sk_to_pk(
                pk.as_mut_ptr(),
                input.as_ptr()
            ); // Always success
        }

        KeyPair {
            sk: PrivateKey::from(input),
            pk: PublicKey::from(&pk)
        }
    }

    pub fn from_seed<'a>(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            KeyPair::SEED_BYTES,
            "Incorrect seed size {}, should be {}",
            input.len(),
            KeyPair::SEED_BYTES
        );

        let mut sk = vec![0u8; PrivateKey::BYTES];
        let mut pk = vec![0u8; PublicKey::BYTES];

        unsafe {
            crypto_sign_seed_keypair(
                pk.as_mut_ptr(),
                sk.as_mut_ptr(),
                input.as_ptr()
            ); // Always success
        }
        KeyPair {
            sk: PrivateKey::from(&sk),
            pk: PublicKey::from(&pk)
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
            sk: PrivateKey::from(&sk),
            pk: PublicKey::from(&pk)
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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SignState(
    [u8; std::mem::size_of::<crypto_sign_state>()]
);

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Signature {
    state: SignState
}

impl Signature {
    pub const BYTES: usize = 64;

    pub fn reset(&mut self) {
        assert!(
            std::mem::size_of::<SignState>() >= std::mem::size_of::<crypto_sign_state>(),
            "Inappropriate signature state size."
        );

        let s = &mut self.state.0 as *mut _ as *mut crypto_sign_state;
        unsafe {
            crypto_sign_init(s);
        }
    }

    pub fn update(&mut self, part: &[u8]) {
        let s = &mut self.state.0 as *mut _ as *mut crypto_sign_state;
        unsafe {
            crypto_sign_update(s,
                part.as_ptr() as *mut libc::c_uchar,
                part.len() as u64
            );
        }
    }

    pub fn sign(&mut self, sign: &mut [u8], sk: &PrivateKey) {
        assert_eq!(
            sign.len(),
            Signature::BYTES,
            "Invalid signature length {}, should be {}",
            sign.len(),
            Signature::BYTES
        );

        let s = &mut self.state.0 as *mut _ as *mut crypto_sign_state;
        unsafe {
            crypto_sign_final_create(s,
                sign.as_ptr() as *mut libc::c_uchar,
                std::ptr::null_mut(),
                sk.as_bytes().as_ptr() as *mut libc::c_uchar
            );
        }
    }

    pub fn sign_into(&mut self, sk: &PrivateKey) -> Vec<u8> {
        let mut signature = vec![0u8; Self::BYTES];
        self.sign(signature.as_mut_slice(), sk);
        signature
    }

    pub fn verify(&mut self, sign: &[u8], pk: &PublicKey) -> bool {
        assert_eq!(
            sign.len(),
            Signature::BYTES,
            "Invalid signature length {}, should be {}",
            sign.len(),
            Signature::BYTES
        );

        let s = &mut self.state.0 as *mut _ as *mut crypto_sign_state;
        unsafe {
            let result = crypto_sign_final_verify(s,
                sign.as_ptr() as *mut libc::c_uchar,
                pk.as_bytes().as_ptr() as *mut libc::c_uchar
            );
            result == 0
        }
    }

    pub fn sign_with(&self, data: &[u8], sk: &PrivateKey) -> Vec<u8> {
        sk.sign_into(data)
    }

    pub fn verify_with(data: &[u8], signature: &[u8], pk: &PublicKey) -> bool {
        pk.verify(data, signature)
    }
}
