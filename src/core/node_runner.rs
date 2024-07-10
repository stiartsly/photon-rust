use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use log::{info, error};

use crate::{
    constants,
    cryptobox,
    Id,
    dht::DHT,
    config::Config,
    data_storage::DataStorage,
    sqlite_storage::SqliteStorage,
    token_man::TokenManager,
    server::{self, Server},
    bootstrap::BootstrapZone,
    crypto_cache::CryptoCache,
};

#[allow(dead_code)]
pub(crate) struct NodeRunner {
    nodeid: Id,
    storage_path: String,

    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,
    dht_num: i32,

    storage: Rc<RefCell<dyn DataStorage>>,
    server: Rc<RefCell<Server>>,
    tokenman: Rc<RefCell<TokenManager>>,

    cloned: Option<Rc<RefCell<NodeRunner>>>,

    bootstrap_zone: Arc<Mutex<BootstrapZone>>,
}

impl NodeRunner {
    pub(crate) fn new(params: (Id, String, Arc<Mutex<BootstrapZone>>)) -> Self {
        Self {
            nodeid: params.0.clone(),
            storage_path: params.1,

            dht4: None,
            dht6: None,
            dht_num: 0,

            storage: Rc::new(RefCell::new(SqliteStorage::new())),
            server: Rc::new(RefCell::new(Server::new(params.0))),
            tokenman: Rc::new(RefCell::new(TokenManager::new())),
            cloned: None,
            bootstrap_zone: Arc::clone(&params.2),
        }
    }

    fn server(&self) -> Rc<RefCell<Server>> {
        Rc::clone(&self.server)
    }

    //fn tokenman(&self) -> Rc<RefCell<TokenManager>> {
    //    Rc::clone(&self.tokenman)
    //}

    fn storage(&self) -> Rc<RefCell<dyn DataStorage>> {
        Rc::clone(&self.storage)
    }

    pub(crate) fn start(&mut self, cfg: Arc<Mutex<Box<dyn Config>>>, keypair: cryptobox::KeyPair, quit: Arc<Mutex<bool>>) {
        if let Some(addr4) = cfg.lock().unwrap().addr4() {
            let mut dht = DHT::new(&self.nodeid, addr4);
            dht.enable_persistence(self.storage_path.clone() + "/dht4.cache");
            self.dht4 = Some(Rc::new(RefCell::new(dht)));
        }

        if let Some(addr6) = cfg.lock().unwrap().addr6() {
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

        let scheduler = self.server.borrow().scheduler();
        let cloned_dht = Rc::clone(self.dht4.as_ref().unwrap());
        let bootstrap_zone = Arc::clone(&self.bootstrap_zone);
        scheduler.borrow_mut().add(move || {
            bootstrap_zone.lock().unwrap().pop_all(|item| {
                cloned_dht.borrow_mut().add_bootstrap_node(item.clone());
            })
        },1000, 1000);

        let ctxts = Rc::new(RefCell::new(CryptoCache::new(&keypair)));
        scheduler.borrow_mut().add(move || {
            ctxts.borrow_mut().handle_expiration();
        }, 2000, constants::EXPIRED_CHECK_INTERVAL);

        if let Some(dht4) = self.dht4.as_ref() {
            dht4.borrow_mut()
                .set_server(self.server())
                .set_storage(self.storage())
                .set_cloned(Rc::clone(&dht4))
                .start();

            info!("Started DHT node on ipv4 address: {}", dht4.borrow().socket_addr());
        }

        if let Some(dht6) = self.dht6.as_ref() {
            dht6.borrow_mut()
                .set_server(self.server())
                .set_storage(self.storage())
                .set_cloned(Rc::clone(&dht6))
                .start();

            info!("Started DHT node on ipv4 address: {}", dht6.borrow().socket_addr());
        }

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
