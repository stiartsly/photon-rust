use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::SystemTime;

use ciborium::value::Value as CVal;
use log::{error, warn, info, debug};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::{
    unwrap,
    Id,
    Node,
    PeerInfo,
    PeerBuilder,
    Config,
    cryptobox,
    CryptoBox,
    signature,
    Error,
    error::Result
};

use crate::core::cbor;
use crate::activeproxy::{
    connection::ProxyConnection
};

const IDLE_CHECK_INTERVAL:      u128 = 60 * 1000;           // 60s
const MAX_IDLE_TIME:            u128 = 5 * 60 * 1000;       // 5 minutes;
const RE_ANNOUNCE_INTERVAL:     u128 = 60 * 60 * 1000;      // 1hour
const HEALTH_CHECK_INTERVAL:    u128 = 10 * 1000;           // 10s

const CACHED_FILE_NAME: &str = "activeproxy.cache";
const MAX_DATA_PACKET_SIZE: usize = 0x7FFF;

#[allow(dead_code)]
pub struct ProxyClient {
    node:               Rc<RefCell<Node>>,

    persist_path:       String,

    session_keypair:    RefCell<cryptobox::KeyPair>,
    server_pk:          RefCell<Option<cryptobox::PublicKey>>,
    box_:               RefCell<Option<CryptoBox>>,

    server_nodeid:      RefCell<Option<Id>>,
    server_peerid:      Id,

    // Active Proxy service provided server.
    server_host:        RefCell<Option<String>>,
    server_port:        RefCell<Option<u16>>,
    server_name:        RefCell<Option<String>>,
    server_addr:        RefCell<Option<SocketAddr>>,

    domain_name:        Option<String>,
    relay_port:         RefCell<u16>,

    // Upstream server for your local service.
    upstream_host:      String,
    upstream_port:      u16,
    upstream_name:      String,
    upstream_addr:      SocketAddr,

    replaystream_failures: i32,
    upstream_failures:  i32,

    rcvbuf:             Vec<u8>,

    max_connections:    usize,
    inflights:          RefCell<usize>,
    connections:        RefCell<HashMap<i32, Rc<RefCell<ProxyConnection>>>>,

    peer_keypair:       Option<signature::KeyPair>,
    peer:               RefCell<Option<PeerInfo>>,

    last_announcepeer_timestamp:RefCell<SystemTime>,

    last_idle_check_timestamp:  RefCell<SystemTime>,
    last_health_check_timestamp:RefCell<SystemTime>,

    server_failures:    RefCell<i32>,
    reconnect_delay:    RefCell<u128>,
    last_reconnect_timestamp:   RefCell<SystemTime>,

    cloned: RefCell<Option<Rc<RefCell<ProxyClient>>>>,
}

#[allow(dead_code)]
impl ProxyClient {
    pub fn new(node: Rc<RefCell<Node>>, cfg: Box<dyn Config>) -> Result<Self> {
        let Some(ap) = cfg.activeproxy() else {
            error!("The ActiveProxy configuration is missing, preventing the use of the ActiveProxy function!!!
                Please check the config file later.");
            return Err(Error::Argument(format!("ActiveProxy configuration is missing")));
        };

        let path = {
            let mut path = String::from(cfg.storage_path());
            if path.is_empty() {
                path.push_str(".")
            }
            if !path.ends_with("/") {
                path.push_str("/");
            }
            path.push_str(CACHED_FILE_NAME);
            path
        };

        let peer_keypair = match ap.peer_private_key() {
            Some(v) => Some(signature::KeyPair::try_from(v.as_bytes())?),
            None => None,
        };

        let upstream_name = format!( "{}:{}", ap.upstream_host(), ap.upstream_port());
        let upstream_addr = match upstream_name.to_socket_addrs() {
            Ok(mut addrs) => addrs.next().unwrap(),
            Err(e) => {
                error!("Failed to resolve the address {} error: {}", upstream_name, e);
                return Err(Error::Argument(format!("Invalid upstream hostname or port")));
            }
        };

        Ok(Self {
            node,
            persist_path:       path,

            session_keypair:    RefCell::new(cryptobox::KeyPair::random()),
            server_pk:          RefCell::new(None),
            box_:               RefCell::new(None),

            server_nodeid:      RefCell::new(None),
            server_peerid:      ap.server_peerid().parse::<Id>()?,

            // The server node that provides active proxy service
            server_host:        RefCell::new(None),
            server_port:        RefCell::new(None),
            server_name:        RefCell::new(None),
            server_addr:        RefCell::new(None),

            domain_name:        ap.domain_name().as_ref().map(|v|v.to_string()),
            relay_port:         RefCell::new(0),

            // The upstream server running your local service.
            upstream_host:      ap.upstream_host().to_string(),
            upstream_port:      ap.upstream_port(),
            upstream_name,
            upstream_addr,

            replaystream_failures: 0,
            upstream_failures:  0,

            rcvbuf:             vec![0u8; MAX_DATA_PACKET_SIZE],

            max_connections:    0,
            inflights:          RefCell::new(0),
            connections:        RefCell::new(HashMap::new()),

            peer_keypair,
            peer:                       RefCell::new(None),
            last_announcepeer_timestamp:RefCell::new(SystemTime::UNIX_EPOCH),

            last_idle_check_timestamp:  RefCell::new(SystemTime::UNIX_EPOCH),
            last_health_check_timestamp:RefCell::new(SystemTime::UNIX_EPOCH),

            server_failures:    RefCell::new(0),
            reconnect_delay:    RefCell::new(0),
            last_reconnect_timestamp:   RefCell::new(SystemTime::UNIX_EPOCH),

            cloned:             RefCell::new(None),
        })
    }
    fn cloned(&self) -> Rc<RefCell<Self>> {
        assert!(self.cloned.borrow().is_some());
        self.cloned.borrow().as_ref().unwrap().clone()
    }

    pub(crate) fn server_nodeid(&self) -> &RefCell<Option<Id>> {
        &self.server_nodeid
    }

    pub(crate) fn srv_host(&self) -> &RefCell<Option<String>> {
        &self.server_host
    }

    pub(crate) fn srv_port(&self) -> &RefCell<Option<u16>> {
        &self.server_port
    }

    pub(crate) fn srv_endp(&self) -> &RefCell<Option<String>> {
        &self.server_name
    }

    pub(crate) fn srv_addr(&self) -> &RefCell<Option<SocketAddr>> {
        &self.server_addr
    }

    pub(crate) fn ups_endp(&self) -> &str {
        self.upstream_host.as_str()
    }

    pub(crate) fn ups_addr(&self) -> &SocketAddr {
        &self.upstream_addr
    }

    pub(crate) fn nodeid(&self) -> &RefCell<Option<Id>> {
        &self.server_nodeid
    }

    pub(crate) fn session_keypair(&self) -> &RefCell<cryptobox::KeyPair> {
        &self.session_keypair
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.server_pk.borrow().is_some()
    }

    pub(crate) const fn allow(&self, _: &SocketAddr) -> bool {
        true
    }

    pub(crate) fn rport(&self) -> u16 {
        *self.relay_port.borrow()
    }

    pub(crate) fn domain_name(&self) -> Option<&str> {
        self.domain_name.as_ref().map(|v|v.as_str())
    }

    // encryption/decryption on session context.
    pub(crate) fn encrypt(&self, plain: &[u8], cipher: &mut [u8], nonce: &cryptobox::Nonce) -> Result<()> {
        unwrap!(self.box_.borrow()).encrypt(plain, cipher, nonce).map(|_|())
    }

    pub(crate) fn decrypt(&self, cipher: &[u8], plain: &mut [u8], nonce: &cryptobox::Nonce) -> Result<()> {
        unwrap!(self.box_.borrow()).decrypt(cipher, plain, nonce).map(|_|())
    }

    // encryption/decryption on Node context.
    pub(crate) fn encrypt_with_node(&self, plain: &[u8], cipher: &mut [u8]) -> Result<()> {
        self.node.borrow().encrypt(
            self.server_nodeid.borrow().as_ref().unwrap(),
            plain,
            cipher
        ).map(|_|())
    }

    pub(crate) fn decrypt_with_node(&self, cipher: &[u8], plain: &mut [u8]) -> Result<()> {
        self.node.borrow().decrypt(
            self.server_nodeid.borrow().as_ref().unwrap(),
            cipher,
            plain
        ).map(|_|())
    }

    pub(crate) fn sign_into_with_node(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.node.borrow().sign_into(data)
    }

    pub(crate) fn set_max_connections(&mut self, connections: usize) {
        self.max_connections = connections;
    }

    pub(crate) fn rcvbuf(&mut self) -> &mut [u8] {
        &mut self.rcvbuf
    }

    fn reset(&self) {
        unimplemented!()
    }

    pub async fn start(&self) -> Result<()> {
        let mut found = false;
        if self.load_peer().is_some() {
            found = true;
        }
        if !found && self.lookup_peer().await.is_some() {
            found = true;
        }

        if !found {
            error!("ActiveProxy client can't find available peer service {} via network.", self.server_peerid);
            return Err(Error::State(format!("Can not find available peer service {}", self.server_peerid)));
        }

        *self.last_idle_check_timestamp.borrow_mut() = SystemTime::now();
        *self.last_health_check_timestamp.borrow_mut() = SystemTime::now();

        // TODO:

        Ok(())
    }

    fn load_peer(&self) -> Option<()> {
        let Ok(mut fp) = File::open(&self.persist_path) else {
            return None;
        };

        let mut buf = vec![];
        if let Err(_) = fp.read_to_end(&mut buf) {
            return None;
        };

        let reader = cbor::Reader::new(&buf);
        let val: CVal = match ciborium::de::from_reader(reader) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let root = val.as_map()?;
        for (k,v) in root {
            let k = k.as_text()?;
            match k {
                "peerId" => {
                    if Id::from_cbor(v)? != self.server_peerid {
                        return None;
                    };
                },
                "serverHost" => {
                    *self.server_host.borrow_mut() = Some(String::from(v.as_text()?))
                },
                "serverPort" => {
                    *self.server_port.borrow_mut() = Some(v.as_integer()?.try_into().unwrap())
                },
                "serverId"   => {
                    *self.server_nodeid.borrow_mut() = Some(Id::from_cbor(v)?)
                },
                _ => return None,
            };
        }

        if self.server_host.borrow().is_none() || self.server_port.borrow().is_none() ||
            self.server_nodeid.borrow().is_none() {
            warn!("The cached peer {} information is invalid, discorded cached data", self.server_peerid);
            return None
        }

        info!("Load peer {} with server {}:{} from persistence file.",
            self.server_peerid, unwrap!(self.server_host.borrow()), unwrap!(self.server_port.borrow()));

        Some(())
    }

    fn save_peer(&self) {
        let val = CVal::Map(vec![
            (
                CVal::Text(String::from("peerId")),
                CVal::Bytes(self.server_peerid.as_bytes().into())
            ),
            (
                CVal::Text(String::from("serverHost")),
                CVal::Text(String::from(unwrap!(self.srv_host().borrow())))
            ),
            (
                CVal::Text(String::from("serverPort")),
                CVal::Integer(self.srv_port().borrow().unwrap().into())
            ),
            (
                CVal::Text(String::from("serverId")),
                CVal::Bytes(unwrap!(self.server_nodeid.borrow()).as_bytes().into())
            )
        ]);

        let mut buf = vec![];
        let writer = cbor::Writer::new(&mut buf);
        let _ = ciborium::ser::into_writer(&val, writer);

        if let Ok(mut fp) =  File::create(&self.persist_path) {
            _ = fp.write_all(&buf);
            _ = fp.sync_data();
        }
    }

    async fn lookup_peer(&self) -> Option<()> {
        info!("ActiveProxy client is trying to find peer {} ...", self.server_peerid);

        let borrowed = self.node.borrow();
        let result = borrowed.find_peer(&self.server_peerid, Some(8), None).await;
        if let Err(e) = result {
            warn!("Trying to find peer on DHT network failed {}, please try it later!!!", e);
            return None;
        }

        let mut peers = result.unwrap();
        if peers.is_empty() {
            warn!("Cannot find a server peer {} at this moment, please try it later!!!", self.server_peerid);
            return None;
        }

        info!("ActiveProxy client found {} peers, extracting server node info...", peers.len());
        peers.shuffle(&mut thread_rng());

        let mut found = false;
        for peer in peers.iter() {
            info!("Trying to lookup node {} hosting peer {} ...", peer.nodeid(), peer.id());

            let result = borrowed.find_node(peer.nodeid(), None).await;
            if let Err(e) = result {
                warn!("ActiveProxy client failed to locate node: {} with error {}", peer.nodeid(), e);
                return None;
            }

            let join_result = result.unwrap();
            if join_result.is_empty() {
                warn!("ActiveProxy client can't locate node: {}! Go on next ...", peer.nodeid());
                continue;
            }

            let mut addr = None;
            if let Some(v6) = join_result.v6() {
                addr = Some(v6.socket_addr().clone());
            }
            if let Some(v4) = join_result.v4() {
                addr = Some(v4.socket_addr().clone());
            }

            *self.server_port.borrow_mut() = Some(peer.port());
            *self.server_nodeid.borrow_mut() = Some(peer.nodeid().clone());
            *self.server_addr.borrow_mut() = Some(addr.unwrap());
            found = true;
            break;
        }

        match found {
            true => Some(()),
            false => None
        }
    }

    async fn announce_peer(&self) {
        let borrowed = self.peer.borrow();
        let Some(peer) = borrowed.as_ref() else {
            return;
        };

        info!("Announce peer {} : {}", peer.id(), peer);

        if let Some(url) = peer.alternative_url() {
            info!("-**- ActiveProxy: peer server: {}:{}, domain: {} -**-",
                unwrap!(self.server_host.borrow()), peer.port(), url);
        } else {
            info!("-**- ActiveProxy: peer server: {}:{} -**-",
                unwrap!(self.server_host.borrow()), peer.port());
        }

        _ = self.node.borrow().announce_peer(peer, None).await;
    }

    async fn idle_check(&self) {
        // Dump the current status: should change the log level to debug later
        debug!("Addon ActiveProxy STATUS dump: Connections = {}, inFlights = {}, idle = {}",
            self.connections.borrow().len(), self.inflights.borrow(),
            unwrap!(self.last_idle_check_timestamp.borrow().elapsed()).as_secs());

        for (_, item) in self.connections.borrow().iter() {
            debug!("ActiveProxy STATUS dump: \n{}", item.borrow());
        }

        if unwrap!(self.last_idle_check_timestamp.borrow().elapsed()).as_millis() < MAX_IDLE_TIME  ||
            *self.inflights.borrow() > 0 || self.connections.borrow().len() <= 1 {
            return;
        }

        info!("ActiveProxy client is closing the redundant connections due to long time idle...");

        let keys: Vec<_> = self.connections.borrow().keys().cloned().collect();
        for key in keys {
            let item = self.connections.borrow_mut().remove(&key).unwrap();
            _ = item.borrow_mut().on_closed().await.map_err(|e| {
                error!("Error during on_closed {:?}", e);
            });
            _ = item.borrow_mut().close().await.map_err(|e| {
                error!("Error during close {:?}", e);
            });
        }
    }

    async fn health_check(&self) {
        for (_, item) in self.connections.borrow().iter() {
            item.borrow_mut().periodic_check();
        }
    }

    async fn on_iteration(&self) {
        if self.needs_new_connection() {
            _ = self.try_connect().await;
        }

        if unwrap!(self.last_idle_check_timestamp.borrow().elapsed()).as_millis() >= IDLE_CHECK_INTERVAL {
            *self.last_idle_check_timestamp.borrow_mut() = SystemTime::now();
            self.idle_check().await;
        }

        if unwrap!(self.last_health_check_timestamp.borrow().elapsed()).as_millis() >= HEALTH_CHECK_INTERVAL {
            *self.last_health_check_timestamp.borrow_mut() = SystemTime::now();
            self.health_check().await;
        }

        if self.peer.borrow().is_some() &&
            unwrap!(self.last_announcepeer_timestamp.borrow().elapsed()).as_millis() >= RE_ANNOUNCE_INTERVAL {
            *self.last_announcepeer_timestamp.borrow_mut() = SystemTime::now();
            self.announce_peer().await;
        }
    }

    async fn try_connect(&self) -> Result<()> {
        debug!("ActiveProxy client started to create a new connectoin.");

        let mut connection = ProxyConnection::new(self.cloned.borrow().as_ref().unwrap().clone());
        let cloned = self.cloned();
        connection.with_on_authorized_cb(Box::new(move |_: &ProxyConnection, server_pk: &cryptobox::PublicKey, port: u16, domain_enabled: bool| {
            let borrowed = cloned.borrow();
            *borrowed.server_pk.borrow_mut() = Some(server_pk.clone());
            *borrowed.relay_port.borrow_mut() = port;

            if let Some(kp) = borrowed.peer_keypair.as_ref() {

                let borrowed_nodeid = borrowed.server_nodeid().borrow();
                let mut builder = PeerBuilder::new(borrowed_nodeid.as_ref().unwrap());
                let has_domain = borrowed.domain_name.is_some();
                if domain_enabled && has_domain {
                    builder.with_alternative_url(borrowed.domain_name.as_ref().map(|v|v.as_str()).unwrap());
                }

                let peer = builder.with_keypair(kp)
                    .with_origin(borrowed.node.borrow().id())
                    .with_port(port)
                    .build();

                if let Some(url) = peer.alternative_url() {
                    info!("-**- ActiveProxy: peer server: {}:{}, domain: {} -**-",
                        unwrap!(borrowed.server_host.borrow()), peer.port(), url);
                } else {
                    info!("-**- ActiveProxy: peer server: {}:{} -**-",
                        unwrap!(borrowed.server_host.borrow()), peer.port());
                }

                *borrowed.peer.borrow_mut() = Some(peer);
                // Will announce this peer in the next iteration if it's effective.
            };
        }));

        let cloned = self.cloned();
        connection.with_on_opened_cb(Box::new(move |_: &ProxyConnection| {
            *cloned.borrow().server_failures.borrow_mut() = 0;
            *cloned.borrow().reconnect_delay.borrow_mut() = 0;
        }));

        let cloned = self.cloned();
        connection.with_on_open_failed_cb(Box::new(move |_: &ProxyConnection| {
            *cloned.borrow().server_failures.borrow_mut() += 1;
            if *cloned.borrow().reconnect_delay.borrow() < 64 {
                *cloned.borrow().reconnect_delay.borrow_mut() = (1 << *cloned.borrow().server_failures.borrow_mut()) * 1000;
            }
        }));

        let cloned = self.cloned();
        connection.with_on_closed_cb(Box::new(move |conn: &ProxyConnection| {
           cloned.borrow().connections.borrow_mut().remove(&conn.id());
        }));

        let cloned = self.cloned();
        connection.with_on_busy_cb(Box::new(move |_| {
            *cloned.borrow().inflights.borrow_mut() += 1;
            *cloned.borrow().last_idle_check_timestamp.borrow_mut() = SystemTime::UNIX_EPOCH;
        }));

        let cloned = self.cloned();
        connection.with_on_idle_cb(Box::new(move |_| {
            *cloned.borrow().inflights.borrow_mut() -= 1;
            if *cloned.borrow().inflights.borrow() == 0 {
                *cloned.borrow().last_idle_check_timestamp.borrow_mut() = SystemTime::now();
            }
        }));

        let conn_rc = Rc::new(RefCell::new(connection));
        self.connections.borrow_mut().insert(conn_rc.borrow().id(), conn_rc.clone());

        *self.last_reconnect_timestamp.borrow_mut() = SystemTime::now();
        conn_rc.clone().borrow_mut().try_connect_server().await
    }

    fn needs_new_connection(&self) -> bool {
        if self.connections.borrow().len() >= self.max_connections {
            return false;
        }

        if unwrap!(self.last_reconnect_timestamp.borrow().elapsed()).as_millis() < *self.reconnect_delay.borrow() {
            return false;
        }

        if self.connections.borrow().is_empty() {
            if self.server_pk.borrow().is_some() {
                self.reset();
            }
            return true;
        }

        if *self.inflights.borrow() == self.connections.borrow().len() {
            return true;
        }

        // TODO: other conditions ?
        false
    }
}
