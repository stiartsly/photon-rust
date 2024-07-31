use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::collections::LinkedList;

use log::{info, error};

use crate::{
    unwrap,
    constants,
    cryptobox,
    Id,
    NodeInfo,
    Compound,
    LookupOption,
    Network,
    dht::DHT,
    config::Config,
    data_storage::DataStorage,
    sqlite_storage::SqliteStorage,
    token_man::TokenManager,
    server::{self, Server},
    crypto_cache::CryptoCache,
    bootstrap_channel::BootstrapChannel,
    future::{
        FindNodeCmd,
        FindValueCmd,
        FindPeerCmd,
        StoreValueCmd,
        AnnouncePeerCmd,
        Command,
    }
};

pub(crate) struct NodeRunner {
    nodeid: Rc<Id>,
    storage_path: String,

    command_channel:    Option<Arc<Mutex<LinkedList<Command>>>>,
    bootstrap_channel:  Option<Arc<Mutex<BootstrapChannel>>>,

    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,
    dht_num: i32,

    tokenman: Rc<RefCell<TokenManager>>,
    storage:  Rc<RefCell<dyn DataStorage>>,
    server:   Rc<RefCell<Server>>,

    cloned: Option<Rc<RefCell<NodeRunner>>>,
}

impl NodeRunner {
    pub(crate) fn new(input_nodeid: Id, input_storage_path: String) -> Self {
        let id = Rc::new(input_nodeid);

        Self {
            nodeid: id.clone(),
            storage_path: input_storage_path,

            command_channel: None,
            bootstrap_channel: None,

            dht4: None,
            dht6: None,
            dht_num: 0,

            storage:    Rc::new(RefCell::new(SqliteStorage::new())),
            tokenman:   Rc::new(RefCell::new(TokenManager::new())),
            server:     Rc::new(RefCell::new(Server::new(id.clone()))),
            cloned: None,
        }
    }

    pub(crate) fn set_cloned(&mut self, runner: &Rc<RefCell<NodeRunner>>) {
        self.cloned = Some(runner.clone());
    }

    pub(crate) fn set_command_channel(&mut self, channel: &Arc<Mutex<LinkedList<Command>>>) {
        self.command_channel = Some(channel.clone());
    }

    pub(crate) fn set_bootstrap_channel(&mut self, channel: &Arc<Mutex<BootstrapChannel>>) {
        self.bootstrap_channel = Some(channel.clone());
    }

    pub(crate) fn start(&mut self, cfg: Arc<Mutex<Box<dyn Config>>>, keypair: cryptobox::KeyPair) {
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
            dht.set_tokenman(&self.tokenman);
            dht.set_cloned(&dht4);
            dht.start(&cfg.bootstrap_nodes());

            info!("Started DHT node on ipv4 address: {}", dht.socket_addr());
        }

        if let Some(dht6) = self.dht6.as_ref() {
            let mut dht = dht6.borrow_mut();

            dht.set_server(&self.server);
            dht.set_storage(&self.storage);
            dht.set_tokenman(&self.tokenman);
            dht.set_cloned(&dht6);
            dht.start(&cfg.bootstrap_nodes());

            info!("Started DHT node on ipv4 address: {}", dht.socket_addr());
        }

        let scheduler = self.server.borrow().scheduler();
        let ctxts = Rc::new(RefCell::new(CryptoCache::new(&keypair)));
        scheduler.borrow_mut().add(move || {
            ctxts.borrow_mut().handle_expiration();
        }, 2000, constants::EXPIRED_CHECK_INTERVAL);

        let channel = unwrap!(self.bootstrap_channel).clone();
        let dht4 = self.dht4.as_ref().map(|v| v.clone());
        let dht6 = self.dht6.as_ref().map(|v| v.clone());
        scheduler.borrow_mut().add(move || {
            let mut channel = channel.lock().unwrap();
            channel.pop_all(|item| {
                let node = Rc::new(item.clone());
                if let Some(dht) = dht4.as_ref() {
                    dht.borrow_mut().add_bootstrap_node(&node);
                }
                if let Some(dht) = dht6.as_ref() {
                    dht.borrow_mut().add_bootstrap_node(&node);
                }
            });
        }, 1, 60);

        let channel = unwrap!(self.command_channel).clone();
        let runner  = unwrap!(self.cloned).clone();
        scheduler.borrow_mut().add(move || {
            let mut channel = channel.lock().unwrap();
            while let Some(cmd) = channel.pop_front() {
                match cmd {
                    Command::FindNode(c) => runner.borrow().find_node(c),
                    Command::FindValue(c) => runner.borrow().find_value(c),
                    Command::FindPeer(c) => runner.borrow().find_peer(c),
                    Command::StoreValue(c) => runner.borrow().store_value(c),
                    Command::AnnouncePeer(c) => runner.borrow().announce_peer(c),
                }
            }
        }, 1, 60);
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

    fn find_node(&self, cmd: Arc<Mutex<FindNodeCmd>>) {
        let found = Rc::new(RefCell::new(Compound::new()));

        if let Some(dht) = self.dht4.as_ref() {
            dht.borrow().node(cmd.lock().unwrap().id()).map(|v| {
                found.borrow_mut().set_value(Network::Ipv6, v);
            });
        }
        if let Some(dht) = self.dht6.as_ref() {
            dht.borrow().node(cmd.lock().unwrap().id()).map(|v| {
                found.borrow_mut().set_value(Network::Ipv6, v);
            });
        }

        if cmd.lock().unwrap().option() == LookupOption::Arbitrary &&
            found.borrow_mut().has_value() {
            cmd.lock().unwrap().complete(Ok(found.borrow().clone()));
            return;
        }

        let completion = Rc::new(RefCell::new(0));
        let cloned_completion = completion.clone();
        let cloned_found = found.clone();
        let num_dhts = self.dht_num;
        let option = cmd.lock().unwrap().option();
        let id = Rc::new(cmd.lock().unwrap().id().clone());

        let complete_fn = Rc::new(RefCell::new(move |ni: Option<NodeInfo> | {
            *cloned_completion.borrow_mut() += 1;
            ni.map(|v| {
                cloned_found.borrow_mut().set_value(Network::of(v.socket_addr()), v);
            });

            if option == LookupOption::Optimistic &&
                cloned_found.borrow().has_value() &&
                *cloned_completion.borrow() >= num_dhts {
                cmd.lock().unwrap().complete(Ok(cloned_found.borrow().clone()));
            }
        }));

        if let Some(dht) = self.dht4.as_ref() {
            dht.borrow().find_node(id.clone(), option, complete_fn.clone());
        }

        if let Some(dht) = self.dht6.as_ref() {
            dht.borrow().find_node(id.clone(), option, complete_fn.clone());
        }
    }

    fn find_value(&self, _: Arc<Mutex<FindValueCmd>>) {
        unimplemented!()
    }

    fn find_peer(&self, _: Arc<Mutex<FindPeerCmd>>) {
        unimplemented!()
    }

    fn store_value(&self, _: Arc<Mutex<StoreValueCmd>>) {
        unimplemented!()
    }

    fn announce_peer(&self, _: Arc<Mutex<AnnouncePeerCmd>>) {
        unimplemented!()
    }
}

pub(crate) fn run_loop(runner: Rc<RefCell<NodeRunner>>,  quit: Arc<Mutex<bool>>) {
    let server = runner.borrow().server.clone();
    let dht4 = unwrap!(runner.borrow().dht4).clone();

    let result = server.borrow_mut().start(dht4.clone());
    match result {
        Ok(_) => {
            _ = server::run_loop(
                server.clone(),
                dht4.clone(),
                quit.clone()
            ).map_err(|err| {
                error!("Unexpected error happened in the loop: {}.", err);
            });
            server.borrow_mut().stop();
            runner.borrow_mut().stop();
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
