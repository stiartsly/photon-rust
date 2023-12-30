use std::{fs::File, io::Write};
use std::io::Read;
use crate::{
    signature::{KeyPair, PrivateKey},
    id::Id
};

pub struct Node {
    key_pair: KeyPair,
    id: Id,

    persistent:bool
}

impl Node {
    pub fn new() -> Self {
        Node {
            key_pair: KeyPair::random(),
            id: Id::random(),
            persistent: false
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    fn load_key(&mut self, key_path: &str) -> Result<(), &'static str> {
        let file = File::open(key_path);
        if file.is_err() {
            return Err("Opening key file failed");
        }

        let mut buf = Vec::new();
        let result = file.unwrap().read_to_end(&mut buf);
        match result {
            Ok(_) => {
                if buf.len() != PrivateKey::BYTES {
                    return Err("Incorrect key data");
                }
            },
            Err(_) => {
                return Err("Read key file failed.");
            }
        }

        self.key_pair = KeyPair::from_private_key_data(buf.as_slice())?;
        Ok(())
    }

    fn init_key(&mut self, key_path: &str) -> Result<(), &'static str> {
        self.key_pair = KeyPair::random();

        let mut file = File::create(key_path);
        if file.is_err() {
            return Err("Creating a key file failed");
        }

        let result = file.unwrap().write_all(self.key_pair.private_key().bytes());
        match result {
            Ok(()) => { Ok(())},
            Err(_) => {
                Err("Writing key file failed")
            }
        }
    }

}
