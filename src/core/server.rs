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
    version,
    constants,
    cryptobox,
    id::{self, Id},
    node_info::NodeInfo,
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
    bootstrap::Bootstrap,
};

use crate::msg::msg::Msg;

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

    bootstrap: Option<Arc<Mutex<Bootstrap>>>,

    stats: RefCell<Stats>,
    calls: RefCell<HashMap<i32, Box<RpcCall>>>,

    queue4: Option<RefCell<LinkedList<Box<dyn Msg>>>>,
    queue6: Option<RefCell<LinkedList<Box<dyn Msg>>>>,

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

            bootstrap: None,

            stats: RefCell::new(Stats::new()),
            calls: RefCell::new(HashMap::new()),

            queue4: None,
            queue6: None,

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

    pub(crate) fn token_man(&self) -> &Rc<RefCell<TokenManager>> {
        &self.token_man
    }

    pub(crate) fn scheduler(&self) -> &Rc<RefCell<Scheduler>> {
        &self.scheduler
    }

    pub(crate) fn set_bootstrap(&mut self, bootstrap: Arc<Mutex<Bootstrap>>) {
        self.bootstrap = Some(bootstrap);
    }

    pub(crate) fn start<T>(&mut self, dht4: Option<T>, dht6: Option<T>) -> Result<(), Error>
    where
        T: Into<Rc<RefCell<DHT>>>
    {
        if let Some(dht) = dht4.map(|dht| dht.into()) {
            self.dht4 = Some(Rc::clone(&dht));
            self.queue4 = Some(RefCell::new(LinkedList::new()));
        }

        if let Some(dht) = dht6.map(|dht| dht.into()) {
            self.dht6 = Some(Rc::clone(&dht));
            self.queue6 = Some(RefCell::new(LinkedList::new()));
        }

        let path = self.store_path.clone() + "/node.db";
        if let Err(err) = self.storage.borrow_mut().open(&path) {
            error!("Attempt to open database storage failed {}", err);
            return Err(err);
        }

        if let Some(dht) = self.dht4.as_ref() {
            let path = self.store_path.clone() + "/dht4.cache";
            dht.borrow_mut().enable_persistence(&path);
            dht.borrow_mut().start();

            info!(
                "Started RPC server on ipv4 address: {}",
                dht.borrow().addr()
            );

            let _dht = Rc::clone(&dht);
            self.scheduler.borrow_mut().add(
                100,
                constants::DHT_UPDATE_INTERVAL,
                move || {
                    _dht.borrow_mut().update();
            });
        }
        if let Some(dht) = self.dht6.as_ref() {
            let path = self.store_path.clone() + "/dht6.cache";
            dht.borrow_mut().enable_persistence(&path);
            dht.borrow_mut().start();

            info!(
                "Started RPC server on ipv6 address: {}",
                dht.borrow().addr()
            );

            let _dht = Rc::clone(&dht);
            self.scheduler.borrow_mut().add(
                100,
                constants::DHT_UPDATE_INTERVAL,
                move || {
                    _dht.borrow_mut().update();
            });
        }

        let ctxts = Rc::clone(&self.crypto_ctx);
        self.scheduler.borrow_mut().add(
            2000,
            crypto_cache::EXPIRED_CHECK_INTERVAL,
            move || {
                ctxts.borrow_mut().handle_expiration();
        });

        let storage = Rc::clone(&self.storage);
        self.scheduler.borrow_mut().add(
            1000,
            constants::RE_ANNOUNCE_INTERVAL,
            move || {
                persistent_announce(&storage);
        });

        Ok(())
    }

    pub(crate) fn run_loop(&mut self, quit: &Arc<Mutex<bool>>) -> io::Result<()> {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();


        let buffer = Rc::new(RefCell::new(Vec::with_capacity(64*1024))) as Rc<RefCell<Vec<u8>>>;

        self.running = true;
        rt.block_on(async move {
            let mut sock4: Option<UdpSocket> = None;
            let mut sock6: Option<UdpSocket> = None;

            if let Some(dht4) = self.dht4.as_ref() {
                sock4 = Some(UdpSocket::bind(dht4.borrow().addr()).await?);
            }
            if let Some(dht6) = self.dht6.as_ref() {
                sock6 = Some(UdpSocket::bind(dht6.borrow().addr()).await?);
            }

            let mut interval = interval_at(
                self.scheduler.borrow().next_time(),
                Duration::from_secs(60*60)
            );
            while self.running {
                tokio::select! {
                    rc1 = self.read_socket(sock4.as_ref(), Rc::clone(&buffer)) => {
                        match rc1 {
                            Ok(data) => println!("Received data on socket1: {:?}", data),
                            Err(err) => eprintln!("Error reading from socket1: {}", err),
                        }
                    }

                    rc2 = self.read_socket(sock6.as_ref(), Rc::clone(&buffer)) => {
                        match rc2 {
                            Ok(data) => println!("Received data on socket2: {:?}", data),
                            Err(err) => eprintln!("Error reading from socket2: {}", err),
                        }
                    }

                    rc3 = self.write_socket(sock4.as_ref(), true) => {
                        match rc3 {
                           Ok(_) => {},
                           Err(err) => eprintln!("Error writing to socket1 {}", err),
                        }
                    }

                    rc4 = self.write_socket(sock6.as_ref(), false) => {
                        match rc4 {
                           Ok(_) => println!("Written data on socket2 "),
                           Err(err) => eprintln!("Error writing to socket2 {}", err),
                        }
                    }

                    _ = interval.tick() => {
                        self.scheduler.borrow_mut().sync_time();
                        self.scheduler.borrow_mut().run();

                        interval.reset_at(self.scheduler.borrow().next_time());
                    }
                }

                if *quit.lock().unwrap() {
                    self.running = false;
                }
                if self.scheduler.borrow().is_updated() {
                    interval.reset_at(self.scheduler.borrow().next_time());
                }
            }
            Ok(())
        })
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

    pub async fn bootstrap(&self, _: &[NodeInfo]) -> Result<(), Error> {
        unimplemented!()
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

    pub(crate) fn send_msg(&mut self, msg: Box<dyn Msg>, ipv4: bool) {
        // Handle associated call if it exists:
        // - Notify Kademlia DHT of being interacting with a neighboring node;
        // - Process some internal state for this RPC call.
        if let Some(mut call) = msg.associated_call() {
            call.dht().borrow_mut().on_send(call.target_id());
            call.send(&self);
        }

        let queue = match ipv4 {
            true => self.queue4.as_mut().unwrap(),
            false => self.queue6.as_mut().unwrap(),
        };

        queue.borrow_mut().push_back(msg);
    }

    pub(crate) fn send_call(&self, _: Box<RpcCall>) {
        unimplemented!()
    }

    fn decrypt_into(&self, _: &Id, _: &[u8]) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    async fn read_socket<'a>(&self, socket: Option<&UdpSocket>, buffer: Rc<RefCell<Vec<u8>>>) -> Result<Option<usize>, io::Error> {
        match socket {
            Some(socket) => {
                let mut buf = buffer.borrow_mut();
                let (size, addr) = socket.recv_from(&mut buf).await?;
                let sender = Id::from_bytes(&buf[.. id::ID_BYTES]);
                let plain = self.decrypt_into(&sender, &buf[id::ID_BYTES .. size - id::ID_BYTES]).map_err(|err| {
                    self.stats.borrow_mut().on_dropped_packet(size);
                    warn!("Decrypt packet error from {}, ignored: len {}, {}", addr, size, err);
                    return None as Option<usize>
                }).unwrap();

                let mut msg = msg::deser(&sender, &addr, &plain).map_err(|err| {
                    self.stats.borrow_mut().on_dropped_packet(size);
                    warn!("Got a wrong packet from {}, ignored. {}", addr, err);
                    return None as Option<usize>
                }).unwrap();

                self.stats.borrow_mut().on_received_bytes(size);
                self.stats.borrow_mut().on_received_msg(&msg);

                msg.with_id(&sender);
                msg.with_addr(&addr);

                debug!("Received {}/{} from {}:[{}] {}", msg.method(), msg.kind(), addr, size, "msg"); // TODO:

                // transaction id should be a non-zero integer as a normal message.
                if msg.kind() != msg::Kind::Error && msg.txid() == 0 {
                    warn!("Reeived a message with invalid transaction id");
                    //self.send_err(msg, ErrorCode::ProtocolError,
                    //    "Received a message with an invalid transaction id, expected a non-zero transaction id");
                    return Ok(None as Option<usize>);
                }

                // just respond to incoming requests, no need to match them to pending requests
                if msg.kind() == msg::Kind::Request {
                    // handle_msg(msg);
                    return Ok(Some(size));
                }

                // check whether it's a response to an outstanding request
                match self.calls.borrow_mut().remove(&msg.txid()) {
                    Some(mut call) => {
                        // message matches transaction ID and origin == destination
                        // we only check the IP address here. the routing table applies more strict checks to also
                        // verify a stable port
                        // TODO:

                        if call.req().addr() == msg.addr() {
                            call.responsed(&msg);
                            msg.with_associated_call(call);

                            // keep processing after checking whether it's a proper response.
                            // handle_msg(msg);
                            return Ok(Some(size));
                        }

                        // request destination did not match response source!!
                        // this happening by chance is exceedingly unlikely
                        // indicates either port-mangling NAT, a multhomed host listening on any-local address or
                        // some kind of attack ignore response
                        warn!("Transaction id matched, socket address did not, ignoring message, request: {} -> response: {}, version: {}",
                            call.req().addr(), msg.addr(), version::formatted_version(msg.version()));

                        if msg.kind() == msg::Kind::Response && self.dht6.is_some() {
                            // this is more likely due to incorrect binding implementation in ipv6. notify peers about that
                            // don't bother with ipv4, there are too many complications

                            // TODO;
                        }

                        // but expect an upcoming timeout if it's really just a misbehaving node
                        call.response_socket_mismatch();
                        call.stall();
                        return Ok(None);
                    },
                    None => {
                        // - it's not a request
                        // - no matched call found
                        // - up-time is high enough that it's not a stray from a restart did not expect this response

                        if msg.kind() == msg::Kind::Response && self.started.elapsed().unwrap().as_secs() > 2*60 {
                            warn!("Cannot find RPC call for {} {}", msg.kind(), msg.txid());

                            // send_error;
                            return Ok(None);
                        }

                        if msg.kind() == msg::Kind::Error {
                            // handle_msg();
                            return Ok(Some(size));
                        }

                        debug!("Ignored message: {}", "msg"); // TODO:
                    }
                }

                Ok(Some(size))
            },
            None => {
                sleep(Duration::MAX).await;
                Err(io::Error::new(io::ErrorKind::NotFound, "unavailable"))
            }
        }
    }

    async fn write_socket(&self, socket: Option<&UdpSocket>, ipv4: bool) -> Result<(), std::io::Error> {
        if socket.is_none() {
            sleep(Duration::MAX).await;
            return Ok(())
        }

        let queue = match ipv4 {
            true  => self.queue4.as_ref().unwrap(),
            false => self.queue6.as_ref().unwrap(),
        };

        let msg = queue.borrow_mut().pop_front();
        if msg.is_none() {
            sleep(Duration::from_millis(500)).await;
            return Ok(())
        }

        let buffer = msg::serialize(unwrap!(msg));

        // TODO:
        _ = unwrap!(socket).send_to(&buffer, unwrap!(msg).addr());
        Ok(())
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
pub(crate) fn start_tweak<T>(server: &Rc<RefCell<Server>>, addrs: (T, T)) -> Result<(), Error>
where
    T: Into<Option<SocketAddr>>
{
    let mut dht4: Option<Rc<RefCell<DHT>>> = None;
    let mut dht6: Option<Rc<RefCell<DHT>>> = None;

    if let Some(addr) = addrs.0.into() {
        dht4 = Some(Rc::new(RefCell::new(DHT::new(server, addr))));
    }
    if let Some(addr) = addrs.1.into() {
        dht6 = Some(Rc::new(RefCell::new(DHT::new(server, addr))));
    }
    server.borrow_mut().start(dht4, dht6)
}
