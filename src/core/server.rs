use std::rc::Rc;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, LinkedList};

use log::{debug, info, warn, error};
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
    scheduler::Scheduler,
    crypto_cache,
    crypto_cache::CryptoCache,
    stats::Stats,
    msg::msg,
    bootstrap::BootstrapZone,
};

use crate::msg::msg::{Msg};

#[allow(dead_code)]
pub(crate) struct Server {
    id: Id,
    store_path: String,

    running: bool,
    started: SystemTime,

    reachable: bool,
    received_msgs: i32,
    msgs_atleast_reachable_check: i32,
    last_reachable_check: SystemTime,

    bootstrap_zone: Option<Arc<Mutex<BootstrapZone>>>,

    stats: RefCell<Stats>,
    calls: RefCell<HashMap<i32, Box<RpcCall>>>,

    option: LookupOption,
    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,
    dht_num: i32,

    scheduler:  Rc<RefCell<Scheduler>>,
    token_man:  Rc<RefCell<TokenManager>>,
    storage:    Rc<RefCell<dyn DataStorage>>,
    crypto_ctx: Rc<RefCell<CryptoCache>>,
}

#[allow(dead_code)]
impl Server {
    pub fn new(id: Id, store_path: String, keypair: cryptobox::KeyPair) -> Self {
        Self {
            id,
            store_path,
            started: SystemTime::UNIX_EPOCH,
            running: false,

            reachable: false,
            received_msgs: 0,
            msgs_atleast_reachable_check: 0,
            last_reachable_check: SystemTime::UNIX_EPOCH,

            bootstrap_zone: None,

            stats: RefCell::new(Stats::new()),
            calls: RefCell::new(HashMap::new()),

            option: LookupOption::Conservative,
            dht4: None,
            dht6: None,
            dht_num: 0,

            scheduler:  Rc::new(RefCell::new(Scheduler::new())),
            token_man:  Rc::new(RefCell::new(TokenManager::new())),
            storage:    Rc::new(RefCell::new(SqliteStorage::new())),
            crypto_ctx: Rc::new(RefCell::new(CryptoCache::new(&keypair))),
        }
    }

    pub(crate) fn with_bootstrap(&mut self, zone: Arc<Mutex<BootstrapZone>>) {
        self.bootstrap_zone = Some(zone)
    }

    pub(crate) fn tokenman(&self) -> &Rc<RefCell<TokenManager>> {
        &self.token_man
    }

    pub(crate) fn scheduler(&self) -> &Rc<RefCell<Scheduler>> {
        &self.scheduler
    }

    pub(crate) fn dht4(&self) -> Option<Rc<RefCell<DHT>>> {
        match self.dht4.as_ref() {
            Some(dht) => Some(Rc::clone(&dht)),
            None => None
        }
    }

    pub(crate) fn dht6(&self) -> Option<Rc<RefCell<DHT>>> {
        match self.dht6.as_ref() {
            Some(dht) => Some(Rc::clone(&dht)),
            None => None
        }
    }

    pub(crate) fn start(&mut self,
        dht4: Option<Rc<RefCell<DHT>>>,
        dht6: Option<Rc<RefCell<DHT>>>
    ) -> Result<(), Error> {
        let path = self.store_path.clone() + "/node.db";
        if let Err(err) = self.storage.borrow_mut().open(&path) {
            error!("Attempt to open database storage failed {}", err);
            return Err(err);
        }

        if let Some(dht) = dht4.as_ref() {
            self.dht4 = Some(Rc::clone(&dht));

            let path = self.store_path.clone() + "/dht4.cache";
            dht.borrow_mut().enable_persistence(&path);
            dht.borrow_mut().start();

            info!("Started RPC server on ipv4 address: {}", dht.borrow().addr());

            let cloned = Rc::clone(&dht);
            self.scheduler.borrow_mut().add(move || {
                cloned.borrow_mut().update();
            }, 100, constants::DHT_UPDATE_INTERVAL);
        }
        if let Some(dht) = dht6.as_ref() {
            self.dht6 = Some(Rc::clone(&dht));

            let path = self.store_path.clone() + "/dht6.cache";
            dht.borrow_mut().enable_persistence(&path);
            dht.borrow_mut().start();

            info!("Started RPC server on ipv6 address: {}", dht.borrow().addr());

            let cloned = Rc::clone(&dht);
            self.scheduler.borrow_mut().add(move || {
                cloned.borrow_mut().update()
            }, 100, constants::DHT_UPDATE_INTERVAL);
        }

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
        if let Some(zone) = self.bootstrap_zone.as_mut() {
            let zone = Arc::clone(&zone);
            let dht4 = self.dht4();
            let dht6 = self.dht6();

            self.scheduler.borrow_mut().add(move || {
                zone.lock().unwrap().pop_all(|item| {
                    if let Some(dht) = dht4.as_ref() {
                        dht.borrow_mut().add_bootstrap(item.clone());
                    }
                    if let Some(dht) = dht6.as_ref() {
                        dht.borrow_mut().add_bootstrap(item);
                    }
                })
            },1000, 1000);
        }

        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        if let Some(dht) = self.dht4.take() {
            info!("Stopped RPC server on ipv4: {}", dht.borrow().addr());
            dht.borrow_mut().stop();
        }
        if let Some(dht) = self.dht6.take() {
            info!("Started RPC server on ipv6: {}", dht.borrow().addr());
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
pub(crate) fn start_tweak(
    server: &Rc<RefCell<Server>>,
    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>
) -> Result<(), Error>
{
    let mut dht4: Option<Rc<RefCell<DHT>>> = None;
    let mut dht6: Option<Rc<RefCell<DHT>>> = None;

    if let Some(addr) = addr4 {
        let dht = Rc::new(RefCell::new(DHT::new(server, addr)));
        dht4 = Some(dht);
    }
    if let Some(addr) = addr6 {
        let dht = Rc::new(RefCell::new(DHT::new(server, addr)));
        dht6 = Some(dht);
    }
    server.borrow_mut().start(dht4, dht6)
}

pub(crate) fn run_loop(
    server: Rc<RefCell<Server>>,
    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,
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
        let mut sock4: Option<UdpSocket> = None;
        let mut sock6: Option<UdpSocket> = None;
        let mut queue4: Option<Rc<RefCell<LinkedList<Box<dyn Msg>>>>> = None;
        let mut queue6: Option<Rc<RefCell<LinkedList<Box<dyn Msg>>>>> = None;

        if let Some(dht) = dht4.as_ref() {
            sock4 = Some(UdpSocket::bind(dht.borrow().addr()).await?);
            queue4 = Some(dht.borrow().queue());
        }
        if let Some(dht) = dht6.as_ref() {
            sock6 = Some(UdpSocket::bind(dht.borrow().addr()).await?);
            queue6 = Some(dht.borrow().queue());
        }

        let mut interval = interval_at(
            server.borrow().scheduler.borrow().next_time(),
            Duration::from_secs(60*60)
        );
        while running {
            tokio::select! {
                data = read_socket(sock4.as_ref(), Rc::clone(&buffer), move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }) => {
                    match data {
                        Ok(_) => {
                            println!("Received data from ipv4 socket.");
                            //unwrap!(dht4).borrow_mut().on_msg(unwrap!(msg))
                        },
                        Err(_) => {},
                    }
                }

                msg = read_socket(sock6.as_ref(), Rc::clone(&buffer), move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }) => {
                    match msg {
                        Ok(_) => {println!("Received data from ipv6 socket.")},
                        Err(_) => {},
                    }
                }

                _ = write_socket(sock4.as_ref(), queue4.as_ref(),  move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }) => {
                    println!("Write data to ipv4 socket");
                }

                _ = write_socket(sock6.as_ref(), queue6.as_ref(),  move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }) => {
                    println!("Write data to ipv6 socket");
                }

                _ = interval.tick() => {
                    server.borrow().scheduler.borrow_mut().sync_time();
                    server.borrow().scheduler.borrow_mut().run();

                    interval.reset_at(server.borrow().scheduler.borrow().next_time());
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
async fn read_socket<F>(
    socket: Option<&UdpSocket>,
    buffer: Rc<RefCell<Vec<u8>>>,
    mut decrypt: F
) -> Result<Option<Box<dyn Msg>>, io::Error>
    where
    F: FnMut(&Id, &mut [u8]) -> Option<Vec<u8>>
{
    if socket.is_none() {
        sleep(Duration::MAX).await;
        return Ok(None)
    }

    let mut buf = buffer.borrow_mut();
    let (size, from_addr) = unwrap!(socket).recv_from(&mut buf).await?;
    let fromid = Id::from_bytes(&buf[.. id::ID_BYTES]);
    let plain = decrypt(&fromid, &mut buf[id::ID_BYTES .. size - id::ID_BYTES]);
    if plain.is_none() {
        //self.stats.borrow_mut().on_dropped_packet(size);
        warn!("Decrypt packet error from {}, ignored: len {}", from_addr, size);
        return Ok(None);
    };

    let mut msg = msg::deser(&fromid, &from_addr, &unwrap!(plain)).map_err(|err| {
        //self.stats.borrow_mut().on_dropped_packet(size);
        warn!("Got a wrong packet from {}, ignored. {}", from_addr, err);
    }).unwrap();

    //self.stats.borrow_mut().on_received_bytes(size);
    //self.stats.borrow_mut().on_received_msg(&msg);

    msg.set_id(&fromid);
    msg.set_addr(&from_addr);

    debug!("Received {}/{} from {}:[{}] {}", msg.method(), msg.kind(), from_addr, size, "msg"); // TODO:

    // transaction id should be a non-zero integer as a normal message.
    if msg.kind() != msg::Kind::Error && msg.txid() == 0 {
        warn!("Reeived a message with invalid transaction id");
        //self.send_err(msg, ErrorCode::ProtocolError,
        //    "Received a message with an invalid transaction id, expected a non-zero transaction id");
        return Ok(None);
    }

    // just respond to incoming requests, no need to match them to pending requests
    if msg.kind() == msg::Kind::Request {
        // handle_msg(msg);
        return Ok(Some(msg));
    }

    Ok(Some(msg))
}

async fn write_socket<F>(
    socket: Option<&UdpSocket>,
    queue: Option<&Rc<RefCell<LinkedList<Box<dyn Msg>>>>>,
    _: F
) -> Result<(), io::Error>
where
    F: FnMut(&Id, &mut [u8]) -> Option<Vec<u8>>
{
    if socket.is_none() || queue.is_none() {
        sleep(Duration::MAX).await;
        return Ok(())
    }

    match unwrap!(queue).borrow_mut().pop_front() {
        Some(msg) => {
            let buffer = msg::serialize(&msg);
            _ = unwrap!(socket).send_to(&buffer, msg.addr());
        },
        None => {
            println!(">>>>>>>>> write_socket");
            sleep(Duration::from_millis(500)).await;
        }
    }
    Ok(())
}
