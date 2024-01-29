use std::{fs::File, io::Write};
use std::rc::{Rc};
use std::io::Read;
use std::boxed::Box;
use log::{info, warn, error};
use std::fs;
use crate::{
    config::Config,
    signature::{self, KeyPair, PrivateKey},
    cryptobox::{self},
    id::Id,
    node_status::NodeStatus,
    value::Value,
    dht::DHT,
    node::Node,
    peer::Peer,
    lookup_option::LookupOption,
    rpcserver::RpcServer,
};

#[allow(dead_code)]
pub struct NodeRunner {
    signature_keypair: signature::KeyPair,
    encryption_keypair: cryptobox::KeyPair,

    id: Id,

    persistent:bool,

    status: NodeStatus,
    cfg: Box<dyn Config>,

    dht4: Option<Box<DHT>>,
    dht6: Option<Box<DHT>>,

    storage_root: String,
}

#[allow(dead_code)]
impl NodeRunner {
    pub fn new(cfg: Box<dyn Config>) -> Result<Self, &'static str> {
        if cfg.ipv4().is_none() && cfg.ipv6().is_none() {
            error!("No valid IPv4 or IPv6 address was specified for the new node.");
            return Err("No listening address");
        }

        info!("Photon node is running in development mode.");

        let mut root = String::from(cfg.storage_path());
        if root.is_empty() {
            root.push_str(".")
        }
        if !root.ends_with("/") {
            root.push_str("/");
        }

        let mut persistent = check_peristence(&root).map_err(|err| {
            error!("{}", err);
            return err;
        })?;

        let mut key_path = root.clone();
        key_path.push_str("key");

        let mut key_pair = KeyPair::random();
        if let Err(_) = fs::metadata(&key_path).map(|metadata| {
            if metadata.is_dir() {
                warn!("Key file path {} is an existing directory. DHT node will not be able to persist node key", key_path);
                persistent = false;
                key_pair = KeyPair::random();
            } else {
                key_pair = load_key(&key_path).map_err(|err| {
                    error!("failed {}", err);
                    return err;
                }).unwrap();
            }
        }) {
            key_pair = KeyPair::random();
            init_key(&key_pair, &key_path)?;
        }

        let id = Id::from_signature_key(&key_pair.public_key());
        if persistent {
            let mut id_path = root.clone();
            id_path.push_str("id");
            write_id_file(&id, &id_path)?;
        }

        info!("Boson kademlia node {}", id);

        Ok(NodeRunner {
            dht4: None,
            dht6: None,
            signature_keypair: key_pair,
            encryption_keypair: cryptobox::KeyPair::new(),

            id,
            persistent,
            status: NodeStatus::Stopped,
            cfg,
            storage_root: root
        })
    }

    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.status != NodeStatus::Stopped {
            return Ok(());
        }

        // self.set_status(NodeStatus::Stopped, NodeStatus::Initializing);
        info!("Photon node {} is starting ...", self.id);

        let server = Rc::new(RpcServer::new());
        if let Some(addr4) = self.cfg.ipv4() {

            let mut dht4 = Box::new(DHT::new(addr4, Rc::clone(&server)));
            if self.persistent {
                dht4.enable_persistence(&format!("{}/dht4.cache", self.storage_root));
                self.dht4 = Some(dht4);
            }
        }
        if let Some(addr6) = self.cfg.ipv6() {
            let mut dht6 = Box::new(DHT::new(addr6, Rc::clone(&server)));
            if self.persistent {
                dht6.enable_persistence(&format!("{}/dht4.cache", self.storage_root));
                self.dht4 = Some(dht6);
            }
        }

        // self.set_status(NodeStatus::Initializing, NodeStatus::Running);

        Ok(())
    }

    /*
    impl Node {
    fn start(&mut self) {


        self.set_status(NodeStatus::Initializing, NodeStatus::Running);

        self.server = Some(Arc::new(RPCServer::new(self.clone(), self.dht4.clone(), self.dht6.clone())));
        let scheduler = self.server.as_ref().unwrap().get_scheduler(); // Change to actual method call
        let mut db_path = PathBuf::from(&self.storage_path);
        db_path.push("node.db");

        self.storage = Some(SqliteStorage::open(db_path, scheduler));

        // Start crypto context loading cache check expiration
        let crypto_contexts_clone = self.crypto_contexts.clone();
        scheduler.add(|| {
            crypto_contexts_clone.lock().unwrap().handle_expiration();
        }, CryptoCache::EXPIRED_CHECK_INTERVAL, CryptoCache::EXPIRED_CHECK_INTERVAL);

        self.server.as_ref().unwrap().start(); // Change to actual method call

        let nodes = self.config.get_bootstrap_nodes();
        if let Some(dht4) = &self.dht4 {
            dht4.lock().unwrap().set_server(self.server.clone());
            dht4.lock().unwrap().set_token_manager(self.token_manager.clone()); // Change to actual method call
            dht4.lock().unwrap().start(nodes);
            self.num_dhts += 1;
        }
        if let Some(dht6) = &self.dht6 {
            dht6.lock().unwrap().set_server(self.server.clone());
            dht6.lock().unwrap().set_token_manager(self.token_manager.clone()); // Change to actual method call
            dht6.lock().unwrap().start(nodes);
            self.num_dhts += 1;
        }

        let persistent_announce_clone = self.clone();
        let job = scheduler.add(|| {
            persistent_announce_clone.persistent_announce();
        }, 60000, Constants::RE_ANNOUNCE_INTERVAL);
        self.scheduled_actions.push(job);
    }

    // Add other methods as needed, including set_status, persistent_announce, etc.
}
    */

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn is_self(&self, id: &Id) -> bool {
        self.id == *id
    }

    pub fn find_node(&self, _: &Id, _: &LookupOption) -> Result<Node, &'static str> {
        unimplemented!()
    }

    pub fn find_value(&self, _: &Id, _: &LookupOption) -> Result<Value, &'static str> {
        unimplemented!()
    }

    pub fn store_value(&self, _: &Value) -> Result<(), &'static str> {
        unimplemented!()
    }

    pub fn find_peer(&self, _: &Id, _: i32, _:& LookupOption) -> Result<Vec<Peer>, &'static str> {
        unimplemented!()
    }

    pub fn announce_peer(&self, _: &Peer) -> Result<(), &'static str> {
        unimplemented!()
    }
}

fn load_key(key_path: &str) -> Result<KeyPair, &'static str> {
    let mut file = File::open(key_path)
        .map_err(|_| "Opening key file failed")?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|_| "Reading key file failed")?;

    if buf.len() != PrivateKey::BYTES {
        return Err("Incorrect key size");
    }

    Ok(KeyPair::from_private_key_bytes(buf.as_slice()))
}

fn init_key(key_pair: &KeyPair, key_path: &str) -> Result<(), &'static str> {
    let mut file = File::create(key_path)
        .map_err(|_| "Creating key file failed")?;

    file.write_all(key_pair.private_key().as_bytes())
        .map_err(|_| "Write key file failed")?;

    Ok(())
}

fn write_id_file(id:&Id, key_path: &str) -> Result<(), &'static str> {
    let mut file = File::create(key_path)
        .map_err(|_| "Creating id file failed")?;

    file.write_all(id.to_string().as_bytes())
        .map_err(|_| "Writing ID file failed")?;

    Ok(())
}

fn check_peristence(_: &str) -> Result<bool, &'static str> {
    // TODO
    Ok(true)
}
