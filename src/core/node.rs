use log::{error, info};
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;
use std::thread::{self, JoinHandle};
use std::{fs, fs::File, io::Write};
use std::sync::{Arc, Mutex};

use crate::unwrap;
use crate::logger;
use crate::signature;
use crate::cryptobox;
use crate::id::Id;
use crate::error::Error;
use crate::config::Config;
use crate::NodeInfo;
use crate::NodeStatus;
use crate::peer::Peer;
use crate::value::Value;
use crate::signature::KeyPair;
use crate::crypto_cache::CryptoCache;
use crate::lookup_option::LookupOption;
use crate::server::{self, Server};
use crate::bootstrap::Bootstrap;

pub struct Node {
    id: Id,
    cfg: Box<dyn Config>, //config for this node.

    signature_keypair: signature::KeyPair,
    encryption_keypair: cryptobox::KeyPair,
    encryption_ctxts: Option<RefCell<CryptoCache>>,

    option: LookupOption,
    status: NodeStatus,
    storage_path: String,

    bootstrap: Arc<Mutex<Bootstrap>>,

    worker: Option<JoinHandle<()>>, // engine working thread.
    quit: Arc<Mutex<bool>>, // notification handle
}

impl Node {
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
                        "Bad file path {}, untable to persist node key",
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
                _ = store_key(keypair.as_ref().unwrap(), &keypath).map_err(|err| {
                    error!("Perisisting key data error {}", err);
                    return err;
                })
            }
        };

        // loading node Id from persistence
        let id = Id::from_signature_key(unwrap!(keypair).public_key());
        let idpath = path.clone() + "id";
        store_nodeid(&id, &idpath).map_err(|err| {
            error!("Persisting node Id data error {}", err);
            return err;
        })?;

        info!("Current DHT node Id {}", id);

        Ok(Node {
            bootstrap: Arc::new(Mutex::new(Bootstrap::from(cfg.bootstrap_nodes()))),
            id,
            cfg,
            signature_keypair: unwrap!(keypair).clone(),
            encryption_keypair: cryptobox::KeyPair::from_signature_keypair(unwrap!(keypair)),
            encryption_ctxts: None,
            status: NodeStatus::Stopped,
            option: LookupOption::Conservative,
            storage_path: path,
            worker: None,
            quit: Arc::new(Mutex::new(false)),
        })
    }

    pub fn start(&mut self) {
        if self.status != NodeStatus::Stopped {
            return;
        }
        self.encryption_ctxts = Some(RefCell::new(CryptoCache::new(&self.encryption_keypair)));
        self.status = NodeStatus::Initializing;
        info!("DHT node {} is starting...", self.id);

        // Parameters used to create the NodeEngine instance:
        // - node id: Unique identifier for the node.
        // - storage path: Path used to save key information for this node.
        // - encryption keypair: Used for encrypting and decrypting incoming and
        //   outgoing messages.
        let params = (
            self.id.clone(),
            self.storage_path.clone(),
            self.encryption_keypair.clone(),
        );

        // Parameters used to run the NodeEngine instance.
        // - addr4: socket ipv4 address
        // - addr6: socket ipv6 address
        let dhts = (
            self.cfg.addr4().clone(),
            self.cfg.addr6().clone()
        );

        // Flag used to signal the spawned thread to stop execution.
        let quit = Arc::clone(&self.quit);
        let bootstrap = Arc::clone(&self.bootstrap);
        self.worker = Some(thread::spawn(move || {
            let server = Rc::new(RefCell::new(
                Server::new(params.0, params.1, params.2)
            ));

            server.borrow_mut().set_bootstrap(bootstrap);
            match server::start_tweak(&server, dhts) {
                Ok(_) => {
                    _ = server.borrow_mut().run_loop(&quit).map_err(|err| {
                        error!("Unexpected error happened in the running loop: {}.", err);
                    });
                    server.borrow_mut().stop();
                },
                Err(err) => {
                    error!("Starting node server error {}, aborted the routine.", err);
                }
            }

            // Need to notify the main thread about any abnormal termination not initiated
            // by the main thread itself.
            let mut _quit = quit.lock().unwrap();
            if !*_quit {
                *_quit = true;
            }
            drop(_quit);
        }));

        self.status = NodeStatus::Running;
    }

    pub fn stop(&mut self) {
        if self.status == NodeStatus::Stopped {
            return;
        }

        info!("DHT node {} stopping...", self.id);

        // Check for abnormal termination in the spawned thread. If the thread is still
        // running, then notify it to abort.
        let mut quit = self.quit.lock().unwrap();
        if !*quit {
            *quit = true;
        }
        drop(quit);

        self.worker.take().unwrap().join().expect("Join worker error");
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

    pub fn set_lookup_option(&mut self, option: LookupOption) {
        self.option = option;
    }

    pub fn lookup_option(&self) -> LookupOption {
        self.option
    }

    pub fn bootstrap(&mut self, nodes: &[NodeInfo]) {
        let mut bootstrap = self.bootstrap.lock().unwrap();
        bootstrap.add_many(nodes);
        drop(bootstrap);
    }

    pub async fn find_node_with_option(
        &self,
        _: &Id,
        _: LookupOption,
    ) -> Result<Option<NodeInfo>, Error> {
        unimplemented!()
    }

    pub async fn find_node(&self, node_id: &Id) -> Result<Option<NodeInfo>, Error> {
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

    pub fn encrypt_into(&self, recipient: &Id, plain: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(unwrap!(self.encryption_ctxts)
            .borrow_mut()
            .get(recipient)
            .encrypt_into(plain)
        )
    }

    pub fn decrypt_into(&self, sender: &Id, cipher: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(unwrap!(self.encryption_ctxts)
            .borrow_mut()
            .get(sender)
            .decrypt_into(cipher)
        )
    }

    pub fn encrypt(&self, recipient: &Id, plain: &[u8], cipher: &mut [u8]) -> Result<(), Error> {
        _ = unwrap!(self.encryption_ctxts)
            .borrow_mut()
            .get(recipient)
            .encrypt(plain, cipher);
        Ok(())
    }

    pub fn decrypt(&self, sender: &Id, cipher: &[u8], plain: &mut [u8]) -> Result<(), Error> {
        _ = unwrap!(self.encryption_ctxts)
            .borrow_mut()
            .get(sender)
            .decrypt(cipher, plain);
        Ok(())
    }

    pub fn sign_into(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(self.signature_keypair.private_key().sign_into(data))
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), Error> {
        self.signature_keypair.public_key().verify(data, signature);
        Ok(())
    }
}

fn load_key(path: &str) -> Result<KeyPair, Error> {
    let mut fp = File::open(path).map_err(|err|
        Error::Io(err, format!("Opening key file failed"))
    )?;

    let mut buf = Vec::new();
    fp.read_to_end(&mut buf).map_err(|err|
        Error::Io(err, format!("Reading key failed"))
    )?;

    if buf.len() != signature::PrivateKey::BYTES {
        return Err(Error::State(format!("Incorrect key size for key data {}", buf.len())));
    }

    Ok(KeyPair::from_private_key_bytes(buf.as_slice()))
}

fn store_key(keypair: &KeyPair, path: &str) -> Result<(), Error> {
    let mut file = File::create(path).map_err(|err|
        Error::Io(err, format!("Creating key file failed"))
    )?;

    file.write_all(keypair.private_key().as_bytes())
        .map_err(|err| Error::Io(err, format!("Writing key failed.")))?;

    Ok(())
}

fn store_nodeid(id: &Id, path: &str) -> Result<(), Error> {
    let mut file = File::create(path).map_err(|err|
        Error::Io(err, format!("Creating Id file failed"))
    )?;

    file.write_all(id.as_bytes())
        .map_err(|err| Error::Io(err, format!("Writing ID failed")))?;

    Ok(())
}

fn check_persistence(_: &str) -> Result<bool, Error> {
    // TODO:
    Ok(false)
}
