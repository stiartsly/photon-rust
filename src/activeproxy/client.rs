use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::net::{SocketAddr, ToSocketAddrs};
use std::fs::File;
use std::path::PathBuf;
use std::io::Read;
use std::thread::{self, JoinHandle};

use ciborium::value::Value as CVal;
use rand::seq::SliceRandom;
use rand::thread_rng;
use log::{error, warn, info};

use crate::core::{
    cbor,
    peer_info::PackBuilder,
    id::MIN_ID,
};

use super::{
    worker::{
        self,
        ProxyWorker,
    },
    inners::InnerFields,
};

use crate::{
    unwrap,
    Id,
    Node,
    PeerInfo,
    Config,
    signature,
    Error,
    error::Result
};

pub struct ProxyClient {
    node:               Arc<Mutex<Node>>,
    cached_dir:         PathBuf,

    remote_peerid:      Id,   // acitve proxy service peer ID.

    upstream_host:      String,
    upstream_port:      u16,
    upstream_endp:      String,
    upstream_addr:      SocketAddr,

    peer_domain:        Option<String>,
    peer_keypair:       RefCell<Option<signature::KeyPair>>,

    worker:             RefCell<Option<JoinHandle<()>>>,
    quit:               RefCell<Arc<Mutex<bool>>>,
}

impl ProxyClient {
    pub fn new(node: Arc<Mutex<Node>>, cfg: &Box<dyn Config>) -> Result<Self> {
        let Some(ap) = cfg.activeproxy() else {
            error!("The configuration for ActiveProxy is missing, preventing the use of the ActiveProxy function!!!
                Please check the config file later.");
            return Err(Error::Argument(format!("ActiveProxy configuration is missing")));
        };

        let cached_dir = {
            let storage_path = cfg.storage_path();
            let mut path = if storage_path.is_empty() {
                PathBuf::from(".")
            } else {
                PathBuf::from(storage_path)
            };

            path.push("activeproxy.cache");
            path
        };

        let keypair = ap.peer_private_key().map(|v| {
            let mut bytes = vec![0u8; signature::PrivateKey::BYTES];
            _ = hex::decode_to_slice(v, &mut bytes[..]).map_err(|e| {
                let err_str = format!("Invalid hexadecimal string as peer private key, error: {e}");
                error!("{err_str}");
                Error::Argument(err_str)
            });

            let sk = signature::PrivateKey::try_from(&bytes[..])
                .map_err(|e| Error::Argument(format!("Invalid private key format: {}", e))).unwrap();

            signature::KeyPair::from(&sk)
        });

        let upstream_name = format!("{}:{}", ap.upstream_host(), ap.upstream_port());
        let upstream_addr = upstream_name.to_socket_addrs()
            .map_err(|e| {
                let err_str = format!("Failed to resolve address '{upstream_name}', error: {e}");
                error!("{err_str}");
                Error::Argument(err_str)
            })?
            .next()
            .ok_or_else(|| {
                let err_str = format!("No valid address found for '{upstream_name}'");
                error!("{err_str}");
                Error::Argument(err_str)
            })?;

        Ok(Self {
            node,
            cached_dir,

            remote_peerid:  ap.server_peerid().parse::<Id>()?,

            upstream_host:  ap.upstream_host().to_string(),
            upstream_port:  ap.upstream_port(),
            upstream_endp:  upstream_name,
            upstream_addr:  upstream_addr,
            peer_domain:    ap.domain_name().map(|v|v.to_string()),
            peer_keypair:   RefCell::new(keypair),

            worker:         RefCell::new(None),
            quit:           RefCell::new(Arc::new(Mutex::new(false))),
        })
    }

    pub fn nodeid(&self) -> Id {
        self.node.lock().unwrap().id().clone()
    }

    pub fn node(&self) -> Arc<Mutex<Node>> {
        self.node.clone()
    }

    pub fn upstream_host(&self) -> &str {
        &self.upstream_host
    }

    pub fn upstream_port(&self) -> u16 {
        self.upstream_port
    }

    pub fn upstream_endpoint(&self) -> &str {
        &self.upstream_endp
    }

    pub fn upstream_addr(&self) -> &SocketAddr {
        &self.upstream_addr
    }

    pub fn domain_name(&self) -> Option<&str> {
        self.peer_domain.as_ref().map(|v|v.as_str())
    }

    pub fn remote_peerid(&self) -> &Id {
        &self.remote_peerid
    }

    pub fn start(&self) -> Result<()> {
        let (peer, addr) = if let Some((peer, addr)) = self.load_peer(&self.remote_peerid) {
            (Some(peer), Some(addr))
        } else {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                self.lookup_peer(&self.remote_peerid).await
                .map(|(peer, addr)| (Some(peer), Some(addr)))
                .unwrap_or((None, None))
            })
        };

        let Some(peer) = peer else {
            error!("No available peers with peer ID {} were found in the DHT network.", self.remote_peerid);
            return Err(Error::State(format!("No available peers with peerid {} found", self.remote_peerid)));
        };

        info!("ActiveProxyClient found the peer serivce: {peer}");

        let node        = self.node.clone();
        let cached_dir  = self.cached_dir.clone();
        let peer_domain = self.peer_domain.clone();
        let peer_keypair= self.peer_keypair.borrow_mut().take();
        let ups_addr    = self.upstream_addr.clone();
        let quit        = self.quit.borrow().clone();

        let worker = thread::spawn(move || {
            let mut inners = InnerFields::new();
            inners.set_remote_peer(peer)
                .set_upstream_domain(peer_domain)
                .set_peer_keypair(peer_keypair)
                .set_remote_addr(addr.unwrap())
                .set_upstream_addr(ups_addr);

            let worker = Rc::new(RefCell::new(ProxyWorker::new(
                node,
                Rc::new(RefCell::new(inners)),
                cached_dir
            )));

            worker.borrow_mut().set_cloned(worker.clone());

            _ = worker::run_loop(worker, quit);
        });

        *self.worker.borrow_mut() = Some(worker);
        Ok(())
    }

    pub fn stop(&self) {
        // TODO:
    }


    fn load_peer(&self, target_id: &Id) -> Option<(PeerInfo, SocketAddr)> {
        let Ok(mut fp) = File::open(&self.cached_dir) else {
            return None;
        };

        let mut buf = vec![];
        let Ok(_) = fp.read_to_end(&mut buf) else {
            return None;
        };

        let reader = cbor::Reader::new(&buf);
        let val: CVal = match ciborium::de::from_reader(reader) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let mut peerid: Option<Id>   = None;
        let mut host: Option<String> = None;
        let mut port: Option<u16>    = None;
        let mut nodeid: Option<Id>   = None;
        let mut sig: Option<Vec<u8>> = None;

        let root = val.as_map()?;
        for (k,v) in root {
            let k = k.as_text()?;
            match k {
                "peerId" =>   peerid = Some(Id::from_cbor(v)?),
                "serverHost" => host = Some(String::from(v.as_text()?)),
                "serverPort" => port = Some(v.as_integer()?.try_into().unwrap()),
                "serverId" => nodeid = Some(Id::from_cbor(v)?),
                "signature" =>   sig = Some(v.as_bytes()?.to_vec()),
                _ => return None,
            };
        }

        if peerid.is_none() || peerid.as_ref().unwrap() != target_id {
            warn!("The peerid {} is outdated, not same as expected: {}, discarded cached data",
            peerid.as_ref().map_or(&MIN_ID, |v|v), target_id);
            return None;
        }

        if host.is_none() || port.is_none() || nodeid.is_none() || sig.is_none() {
            warn!("The cached peer {} information is partly missing, discorded cached data", target_id);
            return None
        }

        let name = format!( "{}:{}", unwrap!(host), unwrap!(port));
        let addr = match name.to_socket_addrs() {
            Ok(mut addrs) => addrs.next().unwrap(),
            Err(e) => {
                error!("Failed to resolve the address {} error: {}", name, e);
                return None;
            }
        };

        let peer = PackBuilder::new(nodeid.unwrap())
            .with_peerid(peerid)
            .with_port(port.unwrap())
            .with_sig(sig)
            .build();

        info!("ActiveProxy loaded peer {} from server {} from persistence file.", peer, name);

        Some((peer, addr))
    }

    async fn lookup_peer(&self, peerid: &Id) -> Option<(PeerInfo, SocketAddr)> {
        info!("ActiveProxyClient is trying to find peer {} ...", peerid);

        let locked = self.node.lock().unwrap();
        let result = locked.find_peer(peerid, Some(4), None).await;
        if let Err(e) = result {
            warn!("Trying to find peer on DHT network but failed with error: {}, please try it later!!!", e);
            return None;
        }

        let mut peers = result.unwrap();
        if peers.is_empty() {
            warn!("No peers with peerid {} is found at this moment, please try it later!!!", peerid);
            return None;
        }

        info!("ActiveProxyClient found {} peers, extracting each node info...", peers.len());
        peers.shuffle(&mut thread_rng());

        while let Some(peer) = peers.pop() {
            let nodeid = peer.nodeid();
            info!("Trying to lookup node {} hosting the peer {} ...", nodeid, peerid);

            let result = locked.find_node(nodeid, None).await;
            if let Err(e) = result {
                warn!("ActiveProxyClient failed to find node {}, error: {}", nodeid, e);
                return None;
            }

            let join_result = result.unwrap();
            if join_result.is_empty() {
                warn!("ActiveProxyClient can't locate node: {}! Go on next ...", nodeid);
                continue;
            }

            let mut ip = None;
            if let Some(v6) = join_result.v6() {
                ip = Some(v6.socket_addr().ip().clone());
            }
            if let Some(v4) = join_result.v4() {
                ip = Some(v4.socket_addr().ip().clone());
            }

            let Some(ip) = ip else {
                continue;
            };

            let addr = SocketAddr::new(ip, peer.port());
            info!("ActiveProxyClient discovered peer {} from server node {} via DHT network.", peer, addr);
            return Some((peer, addr))
        }

        None
    }
}
