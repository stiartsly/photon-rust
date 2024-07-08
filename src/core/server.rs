use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, LinkedList};

use log::{info, warn, error};
use tokio::io;
use tokio::runtime;
use tokio::net::UdpSocket;
use tokio::time::{sleep, interval_at, Duration};

use crate::{
    as_millis,
    constants,
    cryptobox,
    id::{self, Id},
    dht::DHT,
    error::Error,
    rpccall::RpcCall,
    sqlite_storage::SqliteStorage,
    token_man::TokenManager,
    data_storage::DataStorage,
    lookup_option::LookupOption,
    scheduler::{self, Scheduler},
    crypto_cache,
    crypto_cache::CryptoCache,
    msg::msg,
    bootstrap::BootstrapZone,
};

use crate::msg::msg::{Msg};

#[allow(dead_code)]
pub(crate) struct Server<> {
    nodeid: Id,
    store_path: String,

    started: SystemTime,

    reachable: bool,
    received_msgs: i32,
    msgs_atleast_reachable_check: i32,
    last_reachable_check: SystemTime,

    bootstrap_zone: Arc<Mutex<BootstrapZone>>,

    //stats: RefCell<Stats>,
    calls: HashMap<i32, Rc<RefCell<RpcCall>>>,

    option: LookupOption,
    dht4: Option<Rc<RefCell<DHT>>>,
    queue4: Rc<RefCell<LinkedList<Rc<RefCell<dyn Msg>>>>>,

    scheduler:  Rc<RefCell<Scheduler>>,
    token_man:  Rc<RefCell<TokenManager>>,
    storage:    Rc<RefCell<dyn DataStorage>>,
    crypto_ctx: Rc<RefCell<CryptoCache>>
}

#[allow(dead_code)]
impl Server {
    pub fn new(params: (Id, String, cryptobox::KeyPair, Arc<Mutex<BootstrapZone>>)) -> Self {
        Self {
            nodeid: params.0,
            store_path: params.1,
            started: SystemTime::UNIX_EPOCH,

            reachable: false,
            received_msgs: 0,
            msgs_atleast_reachable_check: 0,
            last_reachable_check: SystemTime::UNIX_EPOCH,

            bootstrap_zone: params.3,

            //stats: RefCell::new(Stats::new()),
            calls: HashMap::new(),


            option: LookupOption::Conservative,
            dht4: None,
            queue4: Rc::new(RefCell::new(LinkedList::new())),

            scheduler:  Rc::new(RefCell::new(Scheduler::new())),
            token_man:  Rc::new(RefCell::new(TokenManager::new())),
            storage:    Rc::new(RefCell::new(SqliteStorage::new())),
            crypto_ctx: Rc::new(RefCell::new(CryptoCache::new(&params.2))),
        }
    }

    pub(crate) fn tokenman(&self) -> &Rc<RefCell<TokenManager>> {
        &self.token_man
    }

    pub(crate) fn scheduler(&self) -> Rc<RefCell<Scheduler>> {
        Rc::clone(&self.scheduler)
    }

    pub(crate) fn nodeid(&self) -> &Id {
        &self.nodeid
    }

    pub(crate) fn dht4(&self) -> Rc<RefCell<DHT>> {
        Rc::clone(self.dht4.as_ref().unwrap())
    }

    pub(crate) fn queue4(&self) -> Rc<RefCell<LinkedList<Rc<RefCell<dyn Msg>>>>> {
        Rc::clone(&self.queue4)
    }

    pub(crate) fn storage(&self) -> Rc<RefCell<dyn DataStorage>> {
        Rc::clone(&self.storage)
    }

    pub(crate) fn number_of_acitve_rpc_calls(&self) -> usize {
        self.calls.len()
    }

    pub(crate) fn start(&mut self, dht4: Rc<RefCell<DHT>>) -> Result<(), Error> {
        let path = self.store_path.clone() + "/node.db";
        if let Err(err) = self.storage.borrow_mut().open(&path) {
            error!("Attempt to open database storage failed {}", err);
            return Err(err);
        }

        self.dht4 = Some(Rc::clone(&dht4));
        let path = self.store_path.clone() + "/dht4.cache";
        dht4.borrow_mut().enable_persistence(&path);
        dht4.borrow_mut().start();

        info!("Started RPC server on ipv4 address: {}", dht4.borrow().socket_addr());

        let dht4 = Rc::clone(&dht4);
        self.scheduler.borrow_mut().add(move || {
            dht4.borrow_mut().update();
        }, 100, constants::DHT_UPDATE_INTERVAL);


        let ctxts = Rc::clone(&self.crypto_ctx);
        self.scheduler.borrow_mut().add(move || {
            ctxts.borrow_mut().handle_expiration();
        }, 2000, crypto_cache::EXPIRED_CHECK_INTERVAL);

        let storage = Rc::clone(&self.storage);
        self.scheduler.borrow_mut().add(move || {
            persistent_announce(&storage);
        }, 1000, constants::RE_ANNOUNCE_INTERVAL);

        // A scheduled task to move bootstrap nodes from the outer (user thread)
        // to the internal DHT instance.
        //f let Some(zone) = self.bootstrap_zone() {
        let cloned_zone = Arc::clone(&self.bootstrap_zone);
        let cloned_dht4 = self.dht4();

        self.scheduler.borrow_mut().add(move || {
            cloned_zone.lock().unwrap().pop_all(|item| {
                cloned_dht4.borrow_mut().add_bootstrap_node(item.clone());
            })
        },1000, 1000);

        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        if let Some(dht) = self.dht4.take() {
            info!("Stopped RPC server on ipv4: {}", dht.borrow().socket_addr());
            dht.borrow_mut().stop();
        }

        _ = self.storage.borrow_mut().close();
    }

    pub(crate) fn is_reachable(&self) -> bool {
        self.reachable
    }

    pub(crate) fn update_reachability(&mut self) {
        // Avoid pinging too frequently if we're not receiving any response
        // (the connection might be dead)

        if self.received_msgs != self.msgs_atleast_reachable_check {
            self.reachable = false;
            self.last_reachable_check = SystemTime::now();
            self.msgs_atleast_reachable_check = self.received_msgs;
            return;
        }

        if as_millis!(self.last_reachable_check) >  constants::RPC_SERVER_REACHABILITY_TIMEOUT {
            self.reachable = false;
        }
    }

    fn decrypt_into(&self, _: &Id, _: &[u8]) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    pub(crate) fn send_msg(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        msg.borrow_mut().set_id(self.nodeid());
        if let Some(call) = msg.borrow().associated_call() {
            call.borrow_mut().send();

            let call = Rc::clone(&call);
            self.scheduler.borrow_mut().add(move || {
               call.borrow_mut().check_timeout()
            }, 2000, 10);
        }

        self.queue4.borrow_mut().push_back(msg);
    }

    pub(crate) fn send_call(&mut self, call: Rc<RefCell<RpcCall>>) {
        let msg = call.borrow_mut().req();
        let hashid = call.borrow().hash();
        let cloned = Rc::clone(&call);

        call.borrow_mut().set_responsed_fn(|_,_| {});
        call.borrow_mut().set_timeout_fn(|_call| {
            // self.on_timeout(_call);
        });

        self.calls.insert(call.borrow().hash(), Rc::clone(&call));

        if let Some(msg) = msg {
            msg.borrow_mut().set_txid(hashid);
            msg.borrow_mut().with_associated_call(cloned);
            self.send_msg(msg);
        }
    }

    fn responsed(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        let txid = msg.borrow().txid();
        if let Some(call) = self.calls.remove(&txid) {
            msg.borrow_mut().with_associated_call(Rc::clone(&call));
            call.borrow_mut().responsed(msg)
        }
    }
}

fn persistent_announce(_: &Rc<RefCell<dyn DataStorage>>) {
    info!("Reannounce the perisitent values and peers...");

    // let mut timestamp = SystemTime::now();
    // unimplemented!()
    // TODO:
}

pub(crate) fn run_loop(server: Rc<RefCell<Server>>,
    dht4: Rc<RefCell<DHT>>,
    quit: Arc<Mutex<bool>>
) -> io::Result<()>
{
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let buffer = Rc::new(RefCell::new(vec![0; 64*1024]));
    let mut running = true;

    rt.block_on(async move {
        let sock4 = UdpSocket::bind(dht4.borrow().socket_addr()).await?;
        let queue4 = server.borrow_mut().queue4();

        let mut interval = interval_at(
            server.borrow().scheduler.borrow().next_time(),
            Duration::from_secs(60*60)
        );
        while running {
            tokio::select! {
                data = read_socket(&sock4, Rc::clone(&buffer), move |_, buf| {
                   Ok(buf.to_vec())
                }) => {
                    if let Ok(Some(msg)) = data {
                        server.borrow_mut().responsed(Rc::clone(&msg));
                        dht4.borrow_mut().on_message(msg)
                    }
                }

                _ = write_socket(&sock4, Rc::clone(&dht4), Rc::clone(&queue4),  move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }) => {
                    //println!("Write data to ipv4 socket");
                }

                _ = interval.tick() => {
                    let scheduler = server.borrow().scheduler();
                    scheduler::run_jobs(scheduler);
                }
            }

            if *quit.lock().unwrap() {
                running = false;
            }
            if server.borrow().scheduler.borrow().is_updated() {
                interval.reset_at(server.borrow().scheduler.borrow().next_time());
            }
        }
        Ok(())
    })
}
async fn read_socket<F>(socket: &UdpSocket,
    buffer: Rc<RefCell<Vec<u8>>>,
    mut decrypt: F
) -> Result<Option<Rc<RefCell<dyn Msg>>>, io::Error>
    where F: FnMut(&Id, &mut [u8]) -> Result<Vec<u8>, Error>
{
    let mut buf = buffer.borrow_mut();
    let (len, from) = socket.recv_from(&mut buf).await?;
    let from_id = Id::from_bytes(&buf[0.. id::ID_BYTES]);

    let plain = match decrypt(&from_id, &mut buf[id::ID_BYTES .. len]) {
        Ok(plain) => plain,
        Err(err) => {
            warn!("Decrypt packet from {} error {}, discarded it", err, from);
            return Ok(None);
        }
    };

    let msg = match msg::deser(&plain) {
        Ok(msg) => msg,
        Err(err) => {
            warn!("Got a wrong packet from {} with {}", from, err);
            return Ok(None);
        }
    };

    msg.borrow_mut().set_id(&from_id);
    msg.borrow_mut().set_origin(&from);

    info!("Received message: {}/{} from {}:[size: {}] - {}", msg.borrow().method(), msg.borrow().kind(), from, len, msg.borrow());

    if msg.borrow().kind() != msg::Kind::Error && msg.borrow().txid() == 0 {
        warn!("Received a message with invalid transaction id, ignored it");
        return Ok(None);
    }

    // Just respond to incoming requests, no need to match them to pending requests
    if msg.borrow().kind() == msg::Kind::Request {
        return Ok(Some(msg));
    }

    Ok(Some(msg))
}

async fn write_socket<F>(socket: &UdpSocket,
    dht: Rc<RefCell<DHT>>,
    msg_queue: Rc<RefCell<LinkedList<Rc<RefCell<dyn Msg>>>>>, _: F) -> Result<(), io::Error>
where
    F: FnMut(&Id, &mut [u8]) -> Option<Vec<u8>>
{
    if msg_queue.borrow().is_empty() {
        sleep(Duration::MAX).await;
        return Ok(())
    }

    let msg = match msg_queue.borrow_mut().pop_front() {
        Some(msg) => msg,
        None => {
            sleep(Duration::from_millis(500)).await;
            return Ok(())
        }
    };

    if let Some(call) = msg.borrow().associated_call() {
        dht.borrow_mut().on_send(call.borrow().target_id());
        call.borrow_mut().send();
        // self.scheduler.borrow_mut().add(move || {
        //    call.borrow_mut().check_timeout()
        // }, 2000, 10);
    }

    let serialized = msg::serialize(Rc::clone(&msg));
    let mut buf = Vec::new() as Vec<u8>;

    buf.extend_from_slice(msg.borrow().id().as_bytes());
    buf.extend_from_slice(&serialized);
    _ = socket.send_to(&buf, msg.borrow().remote_addr()).await?;

    Ok(())
}
