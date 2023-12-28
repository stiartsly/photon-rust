#[derive(Debug, Clone, Copy)]
pub struct PrivateKey {
    key: [u8; Self::BYTES]
}

impl PrivateKey {
    const BYTES: usize = 64;

    pub fn new() -> Self {
        PrivateKey {
            key: [0; Self::BYTES]
        }
    }

    pub fn new_with_vec(sk: &Vec<u8>) -> Result<Self, &'static str> {
        if sk.len() != Self::BYTES {
            return Err("Invalid raw private key size");
        }

        match sk.clone().try_into() {
            Ok(array) => Ok(PrivateKey { key: array }),
            Err(_) => {
                return Err("Conversion from Hex to Id failed");
            }
        }
    }

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn clear(&mut self) {
        self.key.fill(0);
    }

    pub fn sign(_: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
        Err("Not implemented yet")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PublicKey {
    key: [u8; Self::BYTES]
}

impl PublicKey {
    const BYTES: usize = 32;

    pub fn new() -> Self {
        PublicKey {
            key: [0; Self::BYTES]
        }
    }

    pub fn size(&self) -> usize {
        Self::BYTES
    }

    pub fn clear(&mut self) {
        self.key.fill(0);
    }

    pub fn verify(_: &Vec<u8>, _: &Vec<u8>) -> Result<bool, &'static str> {
        Err("Method not implemented")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyPair {
    sk: PrivateKey,
    pk:PublicKey
}

impl KeyPair {
    const SEED_BYTES: usize = 32;

    pub fn new() -> Self {
        KeyPair {
            sk: PrivateKey::new(),
            pk: PublicKey::new()
        }
    }

    pub fn new_with_private_key(sk: &PrivateKey) -> Self {
        KeyPair {
            sk: *sk,
            pk: PublicKey::new()
        }
    }

    pub fn new_with_seed(_: &Vec<u8>) -> Self {
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
