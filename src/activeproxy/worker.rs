use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;
use std::time::SystemTime;

use futures::stream::{FuturesUnordered, StreamExt};
use tokio::io::AsyncReadExt;
use tokio::time::Duration;
use ciborium::value::Value as CVal;
use log::{info, debug};

use crate::{
    unwrap,
    Node,
    PeerBuilder,
    cryptobox,
    error::Result,
    core::cbor,
    Error,
};

use crate::activeproxy::{
    connection::ProxyConnection,
    inners::InnerFields
};

const IDLE_CHECK_INTERVAL:      u128 = 60 * 1000;           // 60s
const MAX_IDLE_TIME:            u128 = 5 * 60 * 1000;       // 5 minutes;
const HEALTH_CHECK_INTERVAL:    u128 = 10 * 1000;           // 10s
const RE_ANNOUNCE_INTERVAL:     u128 = 60 * 60 * 1000;      // 1hour
const PERSISTENCE_INTERVAL:     u128 = 60 * 60 * 1000;      // 1hour

pub(crate) struct ProxyWorker {
    node:               Arc<Mutex<Node>>,
    cached_dir:         PathBuf,

    inners:             Rc<RefCell<InnerFields>>,

    inflights:          usize,
    connections:        Rc<RefCell<HashMap<i32, Rc<RefCell<ProxyConnection>>>>>,


    last_announcepeer_timestamp:    SystemTime,

    last_idle_check_timestamp:      SystemTime,
    last_health_check_timestamp:    SystemTime,
    last_save_peer_timestamp:       SystemTime,

    server_failures:    i32,
    reconnect_delay:    u128,
    last_reconnect_timestamp:       SystemTime,

    cloned: Option<Rc<RefCell<ProxyWorker>>>,
}

impl ProxyWorker {
    pub fn new(node: Arc<Mutex<Node>>,
        inners: Rc<RefCell<InnerFields>>,
        cached_dir: PathBuf
    ) -> Self {
        Self {
            node,
            cached_dir,
            inners,

            last_announcepeer_timestamp:SystemTime::UNIX_EPOCH,

           // replaystream_failures: 0,
           // upstream_failures:  0,

            inflights:          0,
            connections:        Rc::new(RefCell::new(HashMap::new())),

            last_idle_check_timestamp:  SystemTime::UNIX_EPOCH,
            last_health_check_timestamp:SystemTime::UNIX_EPOCH,
            last_save_peer_timestamp:   SystemTime::UNIX_EPOCH,

            server_failures:    0,
            reconnect_delay:    0,
            last_reconnect_timestamp:   SystemTime::UNIX_EPOCH,

            cloned:             None,
        }
    }

    pub(crate) fn set_cloned(&mut self, worker: Rc<RefCell<Self>>) {
        self.cloned = Some(worker);
    }

    fn cloned(&self) -> Rc<RefCell<Self>> {
        unwrap!(self.cloned).clone()
    }

    fn inners(&self) -> Rc<RefCell<InnerFields>> {
        self.inners.clone()
    }

    fn reset(&self) {
        unimplemented!()
    }

    fn persist_peer(&self) {
        let inners = self.inners.borrow();
        let val = CVal::Map(vec![
            (
                CVal::Text(String::from("peerId")),
                CVal::Bytes(inners.remote_peerid().as_bytes().into())
            ),
            (
                CVal::Text(String::from("serverHost")),
                CVal::Text(inners.remote_ip().to_string())
            ),
            (
                CVal::Text(String::from("serverPort")),
                CVal::Integer(inners.remote_port().into())
            ),
            (
                CVal::Text(String::from("serverId")),
                CVal::Bytes(inners.remote_nodeid().as_bytes().into())
            ),
            (
                CVal::Text(String::from("signature")),
                CVal::Bytes(inners.remote_peer().signature().into())
            )
        ]);

        let mut buf = vec![];
        let writer = cbor::Writer::new(&mut buf);
        let _ = ciborium::ser::into_writer(&val, writer);

        if let Ok(mut fp) = File::create(&self.cached_dir) {
            _ = fp.write_all(&buf);
            _ = fp.sync_data();
        }
    }

    async fn lookup_peer(&self) {
        // TODO:
    }

    async fn announce_peer(&self) {
        let inners = self.inners.borrow();
        let Some(peer) = inners.upstream_peer() else {
            return;
        };

        info!("Announce peer {} : {}", peer.id(), peer);

        if let Some(url) = peer.alternative_url() {
            info!("-**- ProxyWorker: peer server: {}:{}, domain: {} -**-", inners.remote_ip(), peer.port(), url);
        } else {
            info!("-**- ProxyWorker: peer server: {}:{} -**-", inners.remote_ip(), peer.port());
        }

        _ = self.node.lock()
            .unwrap()
            .announce_peer(peer, None).await;
    }

    async fn idle_check(&mut self) {
        let conns = self.connections.clone();
        // Dump the current status: should change the log level to debug later
        debug!("ProxyWorker STATUS dump: Connections = {}, inFlights = {}, idle = {}",
            conns.borrow().len(), self.inflights,
            unwrap!(self.last_idle_check_timestamp.elapsed()).as_secs());

        for (_, item) in conns.borrow().iter() {
            debug!("ProxyWorker status dump: \n{}", item.borrow());
        }

        if unwrap!(self.last_idle_check_timestamp.elapsed()).as_millis() < MAX_IDLE_TIME  ||
            self.inflights > 0 || conns.borrow().len() <= 1 {
            return;
        }

        info!("ProxyWorker is recycling redundant connections due to long time idle...");

        let keys: Vec<_> = conns.borrow().keys().cloned().collect();
        for key in keys {
            let item = conns.borrow_mut().remove(&key).unwrap();
            item.borrow_mut().on_closed();
            item.borrow_mut().close().await.ok();
        }
    }

    async fn health_check(&mut self) {
        let values: Vec<_> = self.connections.borrow().values().cloned().collect();
        for item in values {
            item.borrow_mut().periodic_check();
        }
    }

    async fn try_connect(&mut self) -> Result<()> {
        debug!("ProxyWorker started to create a new connectoin ...");

        let mut conn = ProxyConnection::new(
            self.node.clone(),
            self.inners.clone()
        );

        conn.with_on_authorized_cb(Box::new(move |conn: &ProxyConnection, pk: &cryptobox::PublicKey, port: u16, domain_enabled: bool| {
            let inners = conn.inners();
            let node   = conn.node();

            let sk = inners.borrow().session_keypair().private_key().clone();

            inners.borrow_mut().set_server_pk(pk.clone());
            inners.borrow_mut().set_relay_port(port);
            inners.borrow_mut().set_cryptobox(pk, &sk);
            inners.borrow_mut().set_domain_enabled(domain_enabled);

            if inners.borrow().peer_keypair().is_none() {
                return;
            }

            let peer = PeerBuilder::new(inners.borrow().remote_nodeid())
                .with_keypair(inners.borrow().peer_keypair())
                .with_origin(Some(node.lock().unwrap().id()))
                .with_alternative_url(inners.borrow().upstream_domain())
                .with_port(port)
                .build();

            if let Some(url) = peer.alternative_url() {
                info!("-**- ActiveProxy: peer server: {}:{}, domain: {} -**-", inners.borrow().remote_ip(), peer.port(), url);
            } else {
                info!("-**- ActiveProxy: peer server: {}:{} -**-", inners.borrow().remote_ip(), peer.port());
            }

            inners.borrow_mut().set_upstream_peer(Some(peer));
            // Will announce this peer in the next iteration if it's effective.
        }));

        let cloned = self.cloned();
        conn.with_on_opened_cb(Box::new(move |_: &ProxyConnection| {
            cloned.borrow_mut().server_failures = 0;
            cloned.borrow_mut().reconnect_delay = 0;
        }));

        let cloned = self.cloned();
        conn.with_on_open_failed_cb(Box::new(move |_: &ProxyConnection| {
            cloned.borrow_mut().server_failures += 1;
            if cloned.borrow().reconnect_delay < 64 {
                cloned.borrow_mut().reconnect_delay = (1 << cloned.borrow_mut().server_failures) * 1000;
            }
        }));

        let cloned = self.connections.clone();
        conn.with_on_closed_cb(Box::new(move |conn: &ProxyConnection| {
           cloned.borrow_mut().remove(&conn.id());
        }));

        let cloned = self.cloned();
        conn.with_on_busy_cb(Box::new(move |_| {
            cloned.borrow_mut().inflights += 1;
            cloned.borrow_mut().last_idle_check_timestamp = SystemTime::UNIX_EPOCH;
        }));

        let cloned = self.cloned();
        conn.with_on_idle_cb(Box::new(move |_| {
            cloned.borrow_mut().inflights -= 1;
            if cloned.borrow().inflights == 0 {
                cloned.borrow_mut().last_idle_check_timestamp = SystemTime::now();
            }
        }));

        let conn = Rc::new(RefCell::new(conn));
        self.connections.borrow_mut().insert(conn.borrow().id(), conn.clone());

        self.last_reconnect_timestamp = SystemTime::now();
        conn.clone().borrow_mut()
            .try_connect_server().await
    }

    fn needs_new_connection(&self) -> bool {
        if self.connections.borrow().len() >= self.inners.borrow().max_connections() {
            return false;
        }
        if unwrap!(self.last_reconnect_timestamp.elapsed()).as_millis() < self.reconnect_delay {
            return false;
        }

        if self.connections.borrow().is_empty() {
            if self.inners.borrow().server_pk().is_some() {
                self.reset()
            }
            return true;
        }
        if self.inflights == self.connections.borrow().len() {
            return true;
        }

        false   // Maybe refine other conditions later.
    }

    async fn on_iteration(&mut self) {
        if self.needs_new_connection() {
            _ = self.try_connect().await;
        }

        if unwrap!(self.last_idle_check_timestamp.elapsed()).as_millis() >= IDLE_CHECK_INTERVAL {
            self.last_idle_check_timestamp = SystemTime::now();
            self.idle_check().await;
        }

        if unwrap!(self.last_health_check_timestamp.elapsed()).as_millis() >= HEALTH_CHECK_INTERVAL {
            self.last_health_check_timestamp = SystemTime::now();
            self.health_check().await;
        }

        if self.inners.borrow().upstream_peer().is_some() &&
            unwrap!(self.last_announcepeer_timestamp.elapsed()).as_millis() >= RE_ANNOUNCE_INTERVAL {
            self.last_announcepeer_timestamp = SystemTime::now();
            self.announce_peer().await;
        }

        if unwrap!(self.last_save_peer_timestamp.elapsed()).as_millis() >= PERSISTENCE_INTERVAL {
            self.last_save_peer_timestamp = SystemTime::now();
            self.lookup_peer().await;
            self.persist_peer();
        }
    }
}

pub(crate) fn run_loop(
    worker: Rc<RefCell<ProxyWorker>>,
    _quit: Arc<Mutex<bool>>
) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let mut interval = tokio::time::interval(
            Duration::from_millis(HEALTH_CHECK_INTERVAL as u64
        ));

        _ = worker.borrow_mut().on_iteration().await;

        loop {
            let mut futures = FuturesUnordered::new();
            let conns: Vec<_> = worker.borrow().connections.borrow().values().cloned().collect();
            for item in conns {
                let item = item.clone();
                let inners = worker.borrow().inners();
                futures.push(async move {
                    let rcvbuf = inners.borrow().rcvbuf();
                    let mut borrowed_rcvbuf = rcvbuf.borrow_mut();

                    let mut borrowed = item.borrow_mut();
                    let result = match borrowed.relay_mut().read(&mut borrowed_rcvbuf.as_mut()).await {
                        Ok(n) if n == 0 => {
                            info!("Connection {} was closed by the server.", borrowed.id());
                            Ok(0)
                        },
                        Ok(len) => {
                            println!(">>>> received {} bytes", len);
                            Ok(len)
                        },
                        Err(e) => {
                            Err(Error::State(format!("Connection {} failed to read server with error: {}", borrowed.id(), e)))
                        }
                    };

                    let len = result.map_err(|e| {
                        panic!("{e}");
                    }).unwrap();


                    _ = borrowed.on_relay_data(&borrowed_rcvbuf.as_ref()[..len]).await;
                });
            }

            tokio::select! {
                result = futures.next() => {
                    match result {
                        Some(_) => {}, //{panic!(">>>>>line:{}", line!())},
                        None => println!("failed"),
                    }
                },

                _ = interval.tick() => {
                    _ = worker.borrow_mut().on_iteration().await;
                }
            }
        }
    })
}
