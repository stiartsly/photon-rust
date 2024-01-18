
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct PublicKey {
}

impl PublicKey {
    pub fn new() -> Self {
        PublicKey {}
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeyPair {
}

#[derive(Clone, Copy, Debug)]
pub struct Nonce {
}

impl Nonce {
    pub fn random() -> Nonce {
        Nonce {}
    }
}

impl KeyPair {
    pub fn new() -> Self {
        KeyPair {}
    }
}