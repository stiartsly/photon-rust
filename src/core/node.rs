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
        let mut file = File::open(key_path)
            .map_err(|_| "Opening key file failed")?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|_| "Reading key file failed")?;

        if buf.len() != PrivateKey::BYTES {
            return Err("Incorrect key data");
        }

        self.key_pair = KeyPair::from_private_key_data(buf.as_slice())?;
        Ok(())
    }

    fn init_key(&mut self, key_path: &str) -> Result<(), &'static str> {
        let mut file = File::create(key_path)
            .map_err(|_| "Creating key file failed")?;

        self.key_pair = KeyPair::random();
        file.write_all(self.key_pair.private_key().as_bytes())
            .map_err(|_| "Write key file failed")?;

        Ok(())
    }

    fn write_id_file(&self, key_path: &str) -> Result<(), &'static str> {
        let mut file = File::create(key_path)
            .map_err(|_| "Creating id file failed")?;

        file.write_all(self.id.to_string().as_bytes())
            .map_err(|_| "Writing ID file failed")?;

        Ok(())
    }

}
