use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use log::{info, error};

use crate::{
    unwrap,
    constants,
    cryptobox,
    Id,
    dht::DHT,
    config::Config,
    data_storage::DataStorage,
    sqlite_storage::SqliteStorage,
    token_man::TokenManager,
    server::{self, Server},
    crypto_cache::CryptoCache,
    bootstrap_cache::BootstrapCache,
};

#[allow(dead_code)]
pub(crate) struct NodeRunner {
    nodeid: Id,
    storage_path: String,

    bootstrap_cache: Option<Arc<Mutex<BootstrapCache>>>,

    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,
    dht_num: i32,

    storage: Rc<RefCell<dyn DataStorage>>,
    server: Rc<RefCell<Server>>,
    tokenman: Rc<RefCell<TokenManager>>,

    cloned: Option<Rc<RefCell<NodeRunner>>>,
}

impl NodeRunner {
    pub(crate) fn new(nodeid: Id, storage_path: String) -> Self {
        Self {
            nodeid: nodeid.clone(),
            storage_path: storage_path,

            bootstrap_cache: None,

            dht4: None,
            dht6: None,
            dht_num: 0,

            storage:    Rc::new(RefCell::new(SqliteStorage::new())),
            tokenman:   Rc::new(RefCell::new(TokenManager::new())),
            server:     Rc::new(RefCell::new(Server::new(nodeid))),
            cloned: None,
        }
    }

    pub(crate) fn set_cloned(&mut self, runner: &Rc<RefCell<NodeRunner>>) {
        self.cloned = Some(Rc::clone(&runner));
    }

    pub(crate) fn set_bootstrap(&mut self, cache: Arc<Mutex<BootstrapCache>>) {
        self.bootstrap_cache = Some(cache)
    }

    pub(crate) fn start(&mut self, cfg: Arc<Mutex<Box<dyn Config>>>, keypair: cryptobox::KeyPair, quit: Arc<Mutex<bool>>) {
        let cfg = cfg.lock().unwrap();

        if let Some(addr4) = cfg.addr4() {
            let mut dht = DHT::new(&self.nodeid, addr4);
            dht.enable_persistence(self.storage_path.clone() + "/dht4.cache");
            self.dht4 = Some(Rc::new(RefCell::new(dht)));
        }

        if let Some(addr6) = cfg.addr6() {
            let mut dht = DHT::new(&self.nodeid, addr6);
            dht.enable_persistence(self.storage_path.clone() + "/dht4.cache");
            self.dht4 = Some(Rc::new(RefCell::new(dht)));
        }

        let path = self.storage_path.clone() + "/node.db";
        if let Err(_) = self.storage.borrow_mut().open(path) {
            // error!("Attempt to open database storage failed {}", err);
            // return Err(err);
            panic!("Attempt to open database storage failed");
        }

        if let Some(dht4) = self.dht4.as_ref() {
            let mut dht = dht4.borrow_mut();

            dht.set_server(&self.server);
            dht.set_storage(&self.storage);
            dht.set_cloned(&dht4);
            dht.start(&cfg.bootstrap_nodes());

            info!("Started DHT node on ipv4 address: {}", dht.socket_addr());
        }

        if let Some(dht6) = self.dht6.as_ref() {
            let mut dht = dht6.borrow_mut();

            dht.set_server(&self.server);
            dht.set_storage(&self.storage);
            dht.set_cloned(&dht6);
            dht.start(&cfg.bootstrap_nodes());

            info!("Started DHT node on ipv4 address: {}", dht.socket_addr());
        }

        let scheduler = self.server.borrow().scheduler();
        let ctxts = Rc::new(RefCell::new(CryptoCache::new(&keypair)));
        scheduler.borrow_mut().add(move || {
            ctxts.borrow_mut().handle_expiration();
        }, 2000, constants::EXPIRED_CHECK_INTERVAL);

        let bcache = Arc::clone(unwrap!(self.bootstrap_cache));
        let dht4 = self.dht4.as_ref().map(|item| Rc::clone(&item));
        let dht6 = self.dht6.as_ref().map(|item| Rc::clone(&item));
        scheduler.borrow_mut().add(move || {
            let mut bcache = bcache.lock().unwrap();
            bcache.pop_all(|item| {
                if let Some(dht) = dht4.as_ref() {
                    dht.borrow_mut().add_bootstrap_node(item.clone());
                }
                if let Some(dht) = dht6.as_ref() {
                    dht.borrow_mut().add_bootstrap_node(item.clone());
                }
            });
        }, 1, 60*10);

        let result = self.server.borrow_mut().start(Rc::clone(self.dht4.as_ref().unwrap()));
        match result {
            Ok(_) => {
                _ = server::run_loop(
                    Rc::clone(&self.server),
                    Rc::clone(self.dht4.as_ref().unwrap()),
                    Arc::clone(&quit),
                ).map_err(|err| {
                    error!("Unexpected error happened in the loop: {}.", err);
                });
                self.server.borrow_mut().stop();
                self.stop();
            },
            Err(err) => {
                error!("Starting node server error {}, aborted.", err);
            }
        }

        // Need to notify the main thread about any abnormal termination not initiated
        // by the main thread itself.
        let mut _quit = quit.lock().unwrap();
        if !*_quit {
            *_quit = true;
        }
        drop(_quit);
    }

    pub(crate) fn stop(&mut self) {
        if let Some(dht4) = self.dht4.as_ref() {
            info!("Started RPC server on ipv4 address: {}", dht4.borrow().socket_addr());
            self.dht4 = None;
        }

        if let Some(dht6) = self.dht6.as_ref() {
            info!("Started RPC server on ipv6 address: {}", dht6.borrow().socket_addr());
            self.dht6 = None;
        }


    }
}
