use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::collections::LinkedList;

use log::{info, error};

use crate::{
    unwrap,
    constants,
    cryptobox,
    signature,
    Id,
    NodeInfo,
    Compound,
    LookupOption,
    Network,
    error::Error,
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

    encryption_keypair: cryptobox::KeyPair,
    encryption_ctx: Rc<RefCell<CryptoCache>>,

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
    pub(crate) fn new(
            keypair: signature::KeyPair,
            storage_path: String,
            config: Arc<Mutex<Box<dyn Config>>>
        ) -> Self {

        let nodeid = Rc::new(Id::from_signature_key(keypair.public_key()));
        let keypair = cryptobox::KeyPair::from_signature_keypair(&keypair);
        let ctx = Rc::new(RefCell::new(CryptoCache::new(&keypair)));

        let cfg = config.lock().unwrap();
        let mut dht_num = 0;
        let dht4 = match cfg.addr4() {
            Some(addr) => {
                let mut dht = DHT::new(&nodeid, addr);
                dht.enable_persistence(storage_path.clone() + "/dht4.cache");
                dht_num += 1;
                Some(dht)
            },
            None => None,
        };

        let dht6 = match cfg.addr6() {
            Some(addr) => {
                let mut dht = DHT::new(&nodeid, addr);
                dht.enable_persistence(storage_path.clone() + "/dht4.cache");
                dht_num += 1;
                Some(dht)
            },
            None => None,
        };

        drop(cfg);

        Self {
            nodeid: nodeid.clone(),
            storage_path: storage_path,

            encryption_keypair: keypair,
            encryption_ctx: ctx,

            command_channel: None,
            bootstrap_channel: None,

            dht4: dht4.map(|v| Rc::new(RefCell::new(v))),
            dht6: dht6.map(|v| Rc::new(RefCell::new(v))),
            dht_num,

            storage:    Rc::new(RefCell::new(SqliteStorage::new())),
            tokenman:   Rc::new(RefCell::new(TokenManager::new())),
            server:     Rc::new(RefCell::new(Server::new(nodeid))),
            cloned: None,
        }
    }

    pub(crate) fn set_cloned(&mut self, runner: Rc<RefCell<NodeRunner>>) {
        self.cloned = Some(runner.clone());
    }

    pub(crate) fn set_command_channel(&mut self, channel: Arc<Mutex<LinkedList<Command>>>) {
        self.command_channel = Some(channel.clone());
    }

    pub(crate) fn set_bootstrap_channel(&mut self, channel: Arc<Mutex<BootstrapChannel>>) {
        self.bootstrap_channel = Some(channel.clone());
    }

    pub(crate) fn start(&mut self) -> Result<(), Error>
    {
        let path = self.storage_path.clone() + "/node.db";
        if let Err(e) = self.storage.borrow_mut().open(path.clone()) {
            return Err(Error::State(format!("Openning data storage {} error: {}", path, e)));
        }

        if let Some(dht) = self.dht4.as_ref() {
            dht.borrow_mut().set_server(&self.server);
            dht.borrow_mut().set_storage(&self.storage);
            dht.borrow_mut().set_tokenman(&self.tokenman);
            dht.borrow_mut().set_cloned(&dht);
            dht.borrow_mut().start().map_err(|e| return e)?;

            info!("Started DHT node on ipv4 address: {}", dht.borrow().socket_addr());
        }

        if let Some(dht) = self.dht6.as_ref() {
            dht.borrow_mut().set_server(&self.server);
            dht.borrow_mut().set_storage(&self.storage);
            dht.borrow_mut().set_tokenman(&self.tokenman);
            dht.borrow_mut().set_cloned(&dht);
            dht.borrow_mut().start().map_err(|e| return e)?;

            info!("Started DHT node on ipv4 address: {}", dht.borrow().socket_addr());
        }

        let scheduler = self.server.borrow().scheduler();
        let ctxts = self.encryption_ctx.clone();
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
        }, 100, constants::DHT_UPDATE_INTERVAL);

        let channel = unwrap!(self.command_channel).clone();
        let runner  = unwrap!(self.cloned).clone();
        scheduler.borrow_mut().add(move || {
            let mut channel = channel.lock().unwrap();
            while let Some(cmd) = channel.pop_front() {
                match cmd {
                    Command::FindNode(c)    => runner.borrow().find_node(c),
                    Command::FindValue(c)   => runner.borrow().find_value(c),
                    Command::FindPeer(c)    => runner.borrow().find_peer(c),
                    Command::StoreValue(c)  => runner.borrow().store_value(c),
                    Command::AnnouncePeer(c)=> runner.borrow().announce_peer(c),
                }
            }
        }, 1, 60);

        Ok(())
    }

    fn stop(&mut self) {
        if let Some(dht) = self.dht4.as_ref() {
            dht.borrow_mut().stop();
            info!("Stopped DHT node on ipv4 address: {}", dht.borrow().socket_addr());
        }

        if let Some(dht) = self.dht6.as_ref() {
            dht.borrow_mut().stop();
            info!("Stopped DHT node on ipv6 address: {}", dht.borrow().socket_addr());
        }

        self.dht6 = None;
        self.dht4 = None;
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

    pub(crate) fn encrypt_into(&self, _: &Id, plain: &[u8]) -> Result<Vec<u8>, Error> {
        /* self.encryption_ctx
            .borrow_mut()
            .get(recipient)
            .encrypt_into(plain) */
        Ok(plain.to_vec())
    }

    pub(crate) fn decrypt_into(&self, _: &Id, cipher: &[u8]) -> Result<Vec<u8>, Error> {
        /* self.encryption_ctx
            .borrow_mut()
            .get(sender)
            .decrypt_into(cipher) */
        Ok(cipher.to_vec())
    }
}

pub(crate) fn run_loop(runner: Rc<RefCell<NodeRunner>>,  quit: Arc<Mutex<bool>>) {
    let server = runner.borrow().server.clone();
    let dht4 = runner.borrow().dht4.as_ref().map(|v| v.clone());
    let dht6 = runner.borrow().dht6.as_ref().map(|v| v.clone());

    let mut to_quit = false;

    server.borrow_mut().start();
    runner.borrow_mut().start().err().map(|e| {
        error!("{}", e);
        to_quit = true;
    });

    if !to_quit {
        _ = server::run_loop(
            runner.clone(),
            server.clone(),
            dht4,
            dht6,
            quit.clone()
        ).map_err(|err| {
            error!("Internal error: {}.", err);
        });
    }

    runner.borrow_mut().stop();
    server.borrow_mut().stop();


    // notify the main thread about any abnormal or normal termination.
    let mut _quit = quit.lock().unwrap();
    if !*_quit {
        *_quit = true;
    }
    drop(_quit);
}
