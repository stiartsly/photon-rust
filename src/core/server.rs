use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, LinkedList};

use log::{info, warn, error};
use tokio::io;
use tokio::runtime;
use tokio::net::UdpSocket;
use tokio::time::{sleep, interval_at, Duration};

use crate::{
    unwrap, as_millis,
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
    stats::Stats,
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

    stats: RefCell<Stats>,
    calls: RefCell<HashMap<i32, Box<RpcCall>>>,

    option: LookupOption,
    dht4: Option<Rc<RefCell<DHT>>>,

    scheduler:  Rc<RefCell<Scheduler>>,
    token_man:  Rc<RefCell<TokenManager>>,
    storage:    Rc<RefCell<dyn DataStorage>>,
    crypto_ctx: Rc<RefCell<CryptoCache>>,
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

            stats: RefCell::new(Stats::new()),
            calls: RefCell::new(HashMap::new()),

            option: LookupOption::Conservative,
            dht4: None,

            scheduler:  Rc::new(RefCell::new(Scheduler::new())),
            token_man:  Rc::new(RefCell::new(TokenManager::new())),
            storage:    Rc::new(RefCell::new(SqliteStorage::new())),
            crypto_ctx: Rc::new(RefCell::new(CryptoCache::new(&params.2))),
        }
    }

    pub(crate) fn tokenman(&self) -> &Rc<RefCell<TokenManager>> {
        &self.token_man
    }

    pub(crate) fn scheduler(&self) -> &Rc<RefCell<Scheduler>> {
        &self.scheduler
    }

    pub(crate) fn nodeid(&self) -> &Id {
        &self.nodeid
    }

    pub(crate) fn dht4(&self) -> Rc<RefCell<DHT>> {
        Rc::clone(self.dht4.as_ref().unwrap())
    }

    pub(crate) fn storage(&self) -> Rc<RefCell<dyn DataStorage>> {
        Rc::clone(&self.storage)
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
}

fn persistent_announce(_: &Rc<RefCell<dyn DataStorage>>) {
    info!("Reannounce the perisitent values and peers...");

    // let mut timestamp = SystemTime::now();
    // unimplemented!()
    // TODO:
}

// Notice: This function aims to resolve the dilemma of circular dependency between
// the "server" instance and the two dht instances, which cannot be resolved by allowing
// use "self" reference in engine method to create dht instances.
pub(crate) fn start_tweak(server: Rc<RefCell<Server>>, addr4: SocketAddr) -> Result<(), Error>
{
    let dht4 = Rc::new(RefCell::new(DHT::new(Rc::clone(&server), addr4)));
    server.borrow_mut().start(dht4)
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
        let queue4 = dht4.borrow().queue();

        let mut interval = interval_at(
            server.borrow().scheduler.borrow().next_time(),
            Duration::from_secs(60*60)
        );
        while running {
            tokio::select! {
                data = read_socket(&sock4, Rc::clone(&buffer), move |_, buf| {
                   Some(buf.to_vec())
                }) => {
                    match data {
                        Ok(mut msg) => {
                            dht4.borrow_mut().on_message(msg.take().unwrap())
                        },
                        Err(_) => {},
                    }
                }

                _ = write_socket(&sock4, Rc::clone(&queue4),  move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }) => {
                    //println!("Write data to ipv4 socket");
                }

                _ = interval.tick() => {
                    scheduler::run_jobs(&server.borrow().scheduler);
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
) -> Result<Option<Box<dyn Msg>>, io::Error>
    where F: FnMut(&Id, &mut [u8]) -> Option<Vec<u8>>
{
    let mut buf = buffer.borrow_mut();
    let (size, from_addr) = socket.recv_from(&mut buf).await?;
    let fromid = Id::from_bytes(&buf[0.. id::ID_BYTES]);
    let plain = decrypt(&fromid, &mut buf[id::ID_BYTES .. size]);
    if plain.is_none() {
        //self.stats.borrow_mut().on_dropped_packet(size);
        warn!("Decrypt packet error from {}, ignored: len {}", from_addr, size);
        return Ok(None);
    };

    let mut msg = msg::deser(&unwrap!(plain)).map_err(|err| {
        //self.stats.borrow_mut().on_dropped_packet(size);
        warn!("Got a wrong packet from {}, ignored. {}", from_addr, err);
    }).unwrap();

    //self.stats.borrow_mut().on_received_bytes(size);
    //self.stats.borrow_mut().on_received_msg(&msg);

    msg.set_id(fromid.clone());
    msg.set_addr(from_addr);

    info!("Received message: {}/{} from {}:[size: {}] {}", msg.method(), msg.kind(), from_addr, size, msg);

    // transaction id should be a non-zero integer as a normal message.
    if msg.kind() != msg::Kind::Error && msg.txid() == 0 {
        warn!("Reeived a message with invalid transaction id");
        // self.send_err(msg, ErrorCode::ProtocolError,
        //    "Received a message with an invalid transaction id, expected a non-zero transaction id");
        return Ok(None);
    }

    // Just respond to incoming requests, no need to match them to pending requests
    if msg.kind() == msg::Kind::Request {
        return Ok(Some(msg));
    }

    Ok(Some(msg))
}

async fn write_socket<F>(socket: &UdpSocket, queue: Rc<RefCell<LinkedList<Box<dyn Msg>>>>, _: F) -> Result<(), io::Error>
where
    F: FnMut(&Id, &mut [u8]) -> Option<Vec<u8>>
{
    if queue.borrow().is_empty() {
        sleep(Duration::MAX).await;
        return Ok(())
    }

    match queue.borrow_mut().pop_front() {
        Some(msg) => {
            let serialized = msg::serialize(&msg);
            let mut buffer = Vec::new() as Vec<u8>;
            buffer.extend_from_slice(msg.id().as_bytes());
            buffer.extend_from_slice(&serialized);
            _ = socket.send_to(&buffer, msg.addr()).await?;
        },
        None => {
            sleep(Duration::from_millis(500)).await;
        }
    }
    Ok(())
}
