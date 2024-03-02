use log::{error, info};
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;
use std::thread::{self, JoinHandle};
use std::{fs, fs::File, io::Write};

use crate::config::Config;
use crate::crypto_cache::CryptoCache;
use crate::cryptobox;
use crate::engine::{self, NodeEngine};
use crate::error::Error;
use crate::id::Id;
use crate::logger;
use crate::lookup_option::LookupOption;
use crate::node::Node;
use crate::node_status::NodeStatus;
use crate::peer::Peer;
use crate::signature;
use crate::signature::KeyPair;
use crate::unwrap;
use crate::value::Value;

pub struct NodeRunner {
    id: Id,
    cfg: Box<dyn Config>, //config for this node.

    keypair: signature::KeyPair, // signature keypair
    encryption_ctxts: RefCell<CryptoCache>,

    option: LookupOption,
    status: NodeStatus,
    storage_path: String,

    worker: Option<JoinHandle<()>>,
}

impl NodeRunner {
    pub fn new(cfg: Box<dyn Config>) -> Result<Self, Error> {
        logger::setup();

        // cfg(DEVELOPMENT)
        info!("Phone node running in development mode!!!");

        // Standardize storage path.
        let mut path = String::from(cfg.storage_path());
        if path.is_empty() {
            path.push_str(".")
        }
        if !path.ends_with("/") {
            path.push_str("/");
        }

        // Loading persistence capability
        check_persistence(&path).map_err(|err| {
            error!("Checking persistence error: {}", err);
            return err;
        })?;

        // Loading key from persistence.
        let keypath = path.clone() + "key";
        let mut keypair = Option::default() as Option<KeyPair>;

        match fs::metadata(&keypath) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    error!(
                        "Key file path {} is an existing directory. DHT node
                        will not be able to persist node key there.",
                        keypath
                    );
                    return Err(Error::State(format!(
                        "Bad ey file path {}, untable to persist node key",
                        keypath
                    )));
                };
                keypair = load_key(&keypath)
                    .map_err(|err| {
                        error!("Loading key data error {}", err);
                        return err;
                    })
                    .ok();
            }
            Err(_) => {
                _ = keypair.insert(KeyPair::random());
                _ = persist_key(keypair.as_ref().unwrap(), &keypath).map_err(|err| {
                    error!("Perisisting key data error {}", err);
                    return err;
                })
            }
        };

        // loading node Id from persistence
        let id = Id::from_signature_key(unwrap!(keypair).public_key());
        let idpath = path.clone() + "id";
        persist_nodeid(&id, &idpath).map_err(|err| {
            error!("Persisting node Id error {}", err);
            return err;
        })?;

        info!("DHT node Id {}", id);

        let encryption_keypair = cryptobox::KeyPair::from_signature_keypair(unwrap!(keypair));
        Ok(NodeRunner {
            id,
            cfg,
            keypair: unwrap!(keypair).clone(),
            encryption_ctxts: RefCell::new(CryptoCache::new(&encryption_keypair)),
            status: NodeStatus::Stopped,
            option: LookupOption::Conservative,
            storage_path: path,
            worker: None,
        })
    }

    pub fn start(&mut self) {
        if self.status != NodeStatus::Stopped {
            return;
        }
        self.status = NodeStatus::Initializing;
        info!("DHT node {} is starting...", self.id);

        let params = (self.id.clone(), self.storage_path.clone());

        let dht_params = (self.cfg.addr4().clone(), self.cfg.addr6().clone());

        self.worker = Some(thread::spawn(move || {
            let engine = Rc::new(RefCell::new(
                NodeEngine::new(params.0, params.1.as_str()).unwrap(),
            ));

            engine::start_tweak(&engine, dht_params.0, dht_params.1);

            _ = engine.borrow_mut().run_loop();
            engine::stop_tweak(&engine);
        }));

        self.status = NodeStatus::Running;
    }

    pub fn stop(&mut self) {
        if self.status == NodeStatus::Stopped {
            return;
        }

        _ = self.worker.take().unwrap().join();

        self.worker = None;
        self.status = NodeStatus::Stopped;
        info!("DHT node {} stopped", self.id);

        logger::teardown();
    }

    pub fn is_running(&self) -> bool {
        self.status == NodeStatus::Running
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn is_self(&self, id: &Id) -> bool {
        &self.id == id
    }

    pub fn set_default_lookup_option(&mut self, option: LookupOption) {
        self.option = option;
    }

    pub fn lookup_option(&self) -> LookupOption {
        self.option
    }

    pub async fn find_node_with_option(
        &self,
        _: &Id,
        _: LookupOption,
    ) -> Result<Option<Node>, Error> {
        unimplemented!()
    }

    pub async fn find_node(&self, node_id: &Id) -> Result<Option<Node>, Error> {
        self.find_node_with_option(node_id, self.option).await
    }

    pub async fn find_value_with_option(
        &self,
        _: &Id,
        _: LookupOption,
    ) -> Result<Option<Value>, Error> {
        unimplemented!()
    }

    pub async fn find_value(&self, value_id: &Id) -> Result<Option<Value>, Error> {
        self.find_value_with_option(value_id, self.option).await
    }

    pub async fn find_peer_with_option(
        &self,
        _: &Id,
        _: i32,
        _: LookupOption,
    ) -> Result<Vec<Peer>, Error> {
        unimplemented!()
    }

    pub async fn find_peer(&self, peer_id: &Id, expected_num: i32) -> Result<Vec<Peer>, Error> {
        self.find_peer_with_option(peer_id, expected_num, self.option)
            .await
    }

    pub async fn store_value_with_persistence(&mut self, _: &Value, _: bool) -> Result<(), Error> {
        unimplemented!()
    }

    pub async fn store_value(&mut self, value: &Value, _: bool) -> Result<(), Error> {
        self.store_value_with_persistence(value, false).await
    }

    pub async fn announce_peer_with_persistence(&mut self, _: &Peer, _: bool) -> Result<(), Error> {
        unimplemented!()
    }

    pub async fn announce_peer(&mut self, peer: &Peer) -> Result<(), Error> {
        self.announce_peer_with_persistence(peer, false).await
    }

    pub fn encrypt_into(&self, recipient: &Id, plain: &[u8]) -> Vec<u8> {
        self.encryption_ctxts
            .borrow_mut()
            .get(recipient)
            .encrypt_into(plain)
    }

    pub fn decrypt_into(&self, sender: &Id, cipher: &[u8]) -> Vec<u8> {
        self.encryption_ctxts
            .borrow_mut()
            .get(sender)
            .decrypt_into(cipher)
    }

    pub fn encrypt(&self, recipient: &Id, plain: &[u8], cipher: &mut [u8]) {
        _ = self
            .encryption_ctxts
            .borrow_mut()
            .get(recipient)
            .encrypt(plain, cipher)
    }

    pub fn decrypt(&self, sender: &Id, cipher: &[u8], plain: &mut [u8]) {
        _ = self
            .encryption_ctxts
            .borrow_mut()
            .get(sender)
            .decrypt(cipher, plain)
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.keypair.private_key().sign_into(data)
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        self.keypair.public_key().verify(data, signature)
    }
}

fn load_key(keypath: &str) -> Result<KeyPair, Error> {
    let mut file =
        File::open(keypath).map_err(|err| Error::Io(err, format!("Opening key file failed")))?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|err| Error::Io(err, format!("Reading key failed")))?;

    match buf.len() == signature::PrivateKey::BYTES {
        true => Ok(KeyPair::from_private_key_bytes(buf.as_slice())),
        false => Err(Error::State(format!("Incorrect key size {}", buf.len()))),
    }
}

fn persist_key(keypair: &KeyPair, keypath: &str) -> Result<(), Error> {
    let mut file =
        File::create(keypath).map_err(|err| Error::Io(err, format!("Creating key file failed")))?;

    file.write_all(keypair.private_key().as_bytes())
        .map_err(|err| Error::Io(err, format!("Writing key failed.")))?;

    Ok(())
}

fn persist_nodeid(id: &Id, keypath: &str) -> Result<(), Error> {
    let mut file =
        File::create(keypath).map_err(|err| Error::Io(err, format!("Creating Id file failed")))?;

    file.write_all(id.as_bytes())
        .map_err(|err| Error::Io(err, format!("Writing ID failed")))?;

    Ok(())
}

fn check_persistence(_: &str) -> Result<bool, Error> {
    // TODO:
    Ok(false)
}
