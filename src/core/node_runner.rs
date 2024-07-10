use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use log::error;

use crate::{
    constants,
    cryptobox,
    Id,
    dht::DHT,
    data_storage::DataStorage,
    sqlite_storage::SqliteStorage,
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
            cloned: None,
            bootstrap_zone: Arc::clone(&params.2),
        }
    }

    pub(crate) fn start(&mut self, addr4: SocketAddr, keypair: cryptobox::KeyPair, quit: Arc<Mutex<bool>>) {
        let path = self.storage_path.clone() + "/node.db";
        if let Err(_) = self.storage.borrow_mut().open(path) {
            // error!("Attempt to open database storage failed {}", err);
            // return Err(err);
            panic!("Attempt to open database storage failed");
        }

        let dht4 = Rc::new(RefCell::new(DHT::new(
            Rc::clone(&self.server),
            Rc::clone(&(self.storage)),
            addr4))
        );

        dht4.borrow_mut().set_cloned(Rc::clone(&dht4));

        let path = self.storage_path.clone() + "/dht4.cache";
        dht4.borrow_mut().enable_persistence(path);
        dht4.borrow_mut().start();

        let scheduler = self.server.borrow().scheduler();
        let cloned_dht = Rc::clone(&dht4);
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

        let result = self.server.borrow_mut().start(Rc::clone(&dht4));
        match result {
            Ok(_) => {
                _ = server::run_loop(
                    Rc::clone(&self.server),
                    Rc::clone(&dht4),
                    Arc::clone(&quit),
                ).map_err(|err| {
                    error!("Unexpected error happened in the loop: {}.", err);
                });
                self.server.borrow_mut().stop();
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
}
