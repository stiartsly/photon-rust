
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

    pub(crate) fn as_bytes(&self) -> &[u8] {
        unimplemented!()
    }
}

impl std::fmt::Display for Nonce {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl KeyPair {
    pub fn new() -> Self {
        KeyPair {}
    }
}