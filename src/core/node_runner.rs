use std::{fs::File, io::Write};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Read;
use std::boxed::Box;
use log::{info, warn, error};
use std::fs;
use crate::{
    unwrap,
    error::Error,
    config::Config,
    signature::{self, KeyPair, PrivateKey},
    cryptobox::{self},
    id::Id,
    node_status::NodeStatus,
    node::Node,
    peer::Peer,
    value::Value,
    dht::DHT,
    lookup_option::LookupOption,
    rpcserver::RpcServer,
    token_man::TokenManager,
    data_storage::DataStorage,
    sqlite_storage::SqliteStorage,
};

#[allow(dead_code)]
pub struct NodeRunner {
    sig_keypair: signature::KeyPair,
    encryption_keypair: cryptobox::KeyPair,
    id: Id,

    // store strategy.
    persistent: bool,
    storage_path: String,

    // kademlia DHT strategy
    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,
    dht_num: i32,
    option: LookupOption,

    status: NodeStatus,

    cfg: Box<dyn Config>,
    token_man: Rc<RefCell<TokenManager>>,
    storage: Rc<RefCell<dyn DataStorage>>,
    server: Rc<RefCell<RpcServer>>,
}

#[allow(dead_code)]
impl NodeRunner {
    pub fn new(cfg: Box<dyn Config>) -> Result<Self, Error> {
        if cfg.ipv4().is_none() && cfg.ipv6().is_none() {
            return Err(Error::Generic(
                format!("No valid IPv4 or IPv6 address was specified.")
            ))
        }

        // TODO:
        info!("Photon node is running in development mode.");

        // Standardize storage root path.
        let mut rootpath = String::from(cfg.storage_path());
        if rootpath.is_empty() {
            rootpath.push_str(".")
        }
        if !rootpath.ends_with("/") {
            rootpath.push_str("/");
        }

        // Loading persistence capability
        let mut persistent = check_peristence(&rootpath).map_err(|err| {
            error!("checking persistence error: {}", err);
            return err;
        })?;

        // Loading key from peristence.
        let keypath = rootpath.clone() + "key";
        let mut keypair = Option::default() as Option<KeyPair>;
        if let Err(_) = fs::metadata(&keypath).map(|metadata| {
            match metadata.is_dir() {
                true => {
                    warn!("Key file path {} is an existing directory. DHT node
                        will not be able to persist node key there.", keypath);
                    persistent = false;
                    keypair = Some(KeyPair::random());
                },
                false => {
                    keypair = load_key(&keypath).map_err(|err| {
                        error!("loading key failed {}", err);
                        return err;
                    }).ok();
                }
            }
        }) {
            keypair = Some(KeyPair::random());
            init_key(keypair.as_ref().unwrap(), &keypath)?;
        }

        let id = Id::from_signature_key(unwrap!(keypair).public_key());
        if persistent {
            let idpath = rootpath.clone() + "id";
            write_id_file(&id, &idpath)?;
        }

        info!("Boson kademlia node Id {}", id);

        Ok(NodeRunner {
            sig_keypair: keypair.take().unwrap(),
            encryption_keypair: cryptobox::KeyPair::new(),
            id,

            persistent,
            storage_path: rootpath,

            dht4: None,
            dht6: None,
            dht_num: 0,
            option: LookupOption::Conservative,

            status: NodeStatus::Stopped,

            cfg,
            token_man: Rc::new(RefCell::new(TokenManager::new())),
            storage: Rc::new(RefCell::new(SqliteStorage::new())),
            //storage:
            server: Rc::new(RefCell::new(RpcServer::new()))
        })
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.status != NodeStatus::Stopped {
            return Ok(());
        }
        self.status = NodeStatus::Initializing;

        info!("DHT node {} is starting...", self.id);

        //let self_ref = Rc::new(RefCell::new(self));
        //self.server.borrow_mut().attach(self_ref);

        if let Some(addr4) = self.cfg.ipv4() {
            let dht4 = Rc::new(RefCell::new(DHT::new(addr4)));
            dht4.borrow_mut().set_rpcserver(Rc::clone(&self.server));
            dht4.borrow_mut().set_token_manager(Rc::clone(&self.token_man));
            dht4.borrow_mut().enable_persistence(self.persistent, &format!("{}/dht4.cache", self.storage_path));
            self.dht4 = Some(Rc::clone(&dht4));
            self.server.borrow_mut().enable_dht4(Rc::clone(&dht4))
        }

        if let Some(addr6) = self.cfg.ipv6() {
            let dht6 = Rc::new(RefCell::new(DHT::new(addr6)));
            dht6.borrow_mut().set_rpcserver(Rc::clone(&self.server));
            dht6.borrow_mut().set_token_manager(Rc::clone(&self.token_man));
            dht6.borrow_mut().enable_persistence(self.persistent, &format!("{}/dht6.cache", self.storage_path));
            self.dht6 = Some(Rc::clone(&dht6));
            self.server.borrow_mut().enable_dht6(Rc::clone(&dht6));
        }

        self.status = NodeStatus::Running;
        let dbpath = self.storage_path.clone() + "/node.db";
        self.storage.borrow_mut().open(dbpath.as_str());

        // TODO:

        Ok(())
    }

    pub fn stop(&mut self) {
        if self.status == NodeStatus::Stopped {
            return;
        }

        info!("DHT node {} is stopping..", self.id);

        self.server.borrow_mut().disable_dht4();
        self.server.borrow_mut().disable_dht6();
        //self.server.stop();

        if let Some(dht4) = self.dht4.as_ref() {
            dht4.borrow_mut().unset_rpcserver();
            dht4.borrow_mut().unset_token_manager();
            dht4.borrow_mut().stop();
        }

        if let Some(dht6) = self.dht6.as_ref() {
            dht6.borrow_mut().unset_rpcserver();
            dht6.borrow_mut().unset_token_manager();
            dht6.borrow_mut().stop();
        }

        // self.server.stop();

        self.status = NodeStatus::Stopped;
        info!("DHT node {} stopped", self.id);
    }

    pub(crate) fn storage(&self) -> Rc<RefCell<dyn DataStorage>> {
        unimplemented!()
    }

    pub fn is_running(&self) -> bool {
        unimplemented!()
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

    pub async fn bootstrap(&self, _: &[Node]) -> Result<(), Error> {
        unimplemented!()
    }

    fn persistent_announce(&mut self) {
        info!("Reannounce the perisitent values and peers...");

        // let mut timestamp = SystemTime::now();
        unimplemented!()
    }

    pub async fn find_node_with_option(&self, _: &Id, _: LookupOption) -> Result<Option<Node>, Error> {
        unimplemented!()
    }

    pub async fn find_node(&self, node_id: &Id) -> Result<Option<Node>, Error> {
        self.find_node_with_option(node_id, self.option).await
    }

    pub async fn find_value_with_option(&self, _: &Id, _: LookupOption) -> Result<Option<Value>, Error> {
        unimplemented!()
    }

    pub async fn find_value(&self, value_id: &Id) -> Result<Option<Value>, Error> {
        self.find_value_with_option(value_id, self.option).await
    }

    pub async fn find_peer_with_option(&self, _: &Id, _: i32, _: LookupOption) -> Result<Vec<Peer>, Error> {
        unimplemented!()
    }

    pub async fn find_peer(&self, peer_id: &Id, expected_num: i32) -> Result<Vec<Peer>, Error> {
        self.find_peer_with_option(peer_id, expected_num, self.option).await
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
}

fn load_key(keypath: &str) -> Result<KeyPair, Error> {
    let mut file = File::open(keypath).map_err(|err|
        Error::Io(err, format!("Opening key file failed"))
    )?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|err|
        Error::Io(err, format!("Reading key failed"))
    )?;

    match buf.len() != PrivateKey::BYTES {
        true => {
            Ok(KeyPair::from_private_key_bytes(buf.as_slice()))
        }
        false => {
            Err(Error::State(format!("Incorrect key size")))
        }
    }
}

fn init_key(keypair: &KeyPair, keypath: &str) -> Result<(), Error> {
    let mut file = File::create(keypath).map_err(|err|
        Error::Io(err, format!("Creating key file failed"))
    )?;

    file.write_all(keypair.private_key().as_bytes()).map_err(|err|
        Error::Io(err, format!("Writing key failed."))
    )?;

    Ok(())
}

fn write_id_file(id:&Id, keypath: &str) -> Result<(), Error> {
    let mut file = File::create(keypath).map_err(|err|
        Error::Io(err, format!("Creating Id file failed"))
    )?;

    file.write_all(id.as_bytes()).map_err(|err|
        Error::Io(err, format!("Writing ID failed"))
    )?;

    Ok(())
}

fn check_peristence(_: &str) -> Result<bool, Error> {
    unimplemented!()
}
