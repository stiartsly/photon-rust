use std::rc::Rc;
use std::cell::RefCell;
use std::net::{IpAddr, SocketAddr};


use crate::{
    unwrap,
    Id,
    PeerInfo,
    cryptobox,
    signature,
};

const MAX_DATA_PACKET_SIZE:     usize = 0x7FFF;

pub(crate) struct InnerFields {
    session_keypair:    Option<cryptobox::KeyPair>,
    server_pk:          Option<cryptobox::PublicKey>,
    crypto_box:         Option<cryptobox::CryptoBox>,

    remote_nodeid:      Option<Id>,
    remote_peerid:      Option<Id>,

    remote_addr:        Option<SocketAddr>,
    remote_name:        Option<String>,
    remote_peer:        Option<PeerInfo>,

    upstream_addr:      Option<SocketAddr>,
    upstream_name:      Option<String>,

    peer_keypair:       Option<signature::KeyPair>,
    domain_enabled:     bool,
    peer_domain:        Option<String>,
    peer:               Option<PeerInfo>,
    relay_port:         Option<u16>,

    rcvbuf:             Rc<RefCell<Box<Vec<u8>>>>,

    max_connections:    usize,
}

impl InnerFields {
    pub(crate) fn new() -> Self {
        Self {
            session_keypair:    Some(cryptobox::KeyPair::random()),
            server_pk:          None,
            crypto_box:         None,

            remote_nodeid:      None,
            remote_peerid:      None,

            remote_addr:        None,
            remote_name:        None,
            remote_peer:        None,

            upstream_addr:      None,
            upstream_name:      None,

            peer_keypair:       None,
            domain_enabled:     false,
            peer_domain:        None,
            peer:               None,
            relay_port:         None,

            rcvbuf:             Rc::new(RefCell::new(Box::new(vec![0u8; MAX_DATA_PACKET_SIZE]))),

            max_connections:    12,
        }
    }

    pub(crate) fn set_server_pk(&mut self, pk: cryptobox::PublicKey) -> &mut Self {
        self.server_pk = Some(pk);
        self
    }

    pub(crate) fn set_cryptobox(&mut self, pk: &cryptobox::PublicKey, sk: &cryptobox::PrivateKey) -> &mut Self {
        self.crypto_box = cryptobox::CryptoBox::try_from((pk, sk)).ok();
        self
    }

    pub(crate) fn set_remote_addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.remote_addr    = Some(addr);
        self.remote_name    = Some(addr.to_string());
        self
    }

    pub(crate) fn set_remote_peer(&mut self, peer: PeerInfo) -> &mut Self {
        self.remote_nodeid  = Some(peer.nodeid().clone());
        self.remote_peerid  = Some(peer.id().clone());
        self.remote_peer    = Some(peer);
        self
    }

    pub(crate) fn set_upstream_addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.upstream_name  = Some(addr.to_string());
        self.upstream_addr  = Some(addr);
        self
    }

    pub(crate) fn set_peer_keypair(&mut self, keypair: Option<signature::KeyPair>) -> &mut Self {
        self.peer_keypair = keypair;
        self
    }

    pub(crate) fn set_domain_enabled(&mut self, enabled: bool) -> &mut Self {
        self.domain_enabled = enabled;
        self
    }

    pub(crate) fn set_upstream_domain(&mut self, domain: Option<String>) -> &mut Self {
        self.peer_domain = domain;
        self
    }

    pub(crate) fn set_upstream_peer(&mut self, peer: Option<PeerInfo>) -> &mut Self {
        self.peer = peer;
        self
    }

    pub(crate) fn set_relay_port(&mut self, port: u16) -> &mut Self {
        self.relay_port = Some(port);
        self
    }

    pub(crate) fn set_max_connections(&mut self, connections: usize) -> &mut Self {
        self.max_connections = connections;
        self
    }

    pub(crate) fn session_keypair(&self) -> &cryptobox::KeyPair {
        unwrap!(self.session_keypair)
    }

    pub(crate) fn server_pk(&self) -> Option<&cryptobox::PublicKey> {
        self.server_pk.as_ref()
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.server_pk.is_some()
    }

    pub(crate) fn cryptobox(&self) -> &cryptobox::CryptoBox {
        unwrap!(self.crypto_box)
    }

    pub(crate) fn remote_nodeid(&self) -> &Id {
        unwrap!(self.remote_nodeid)
    }

    pub(crate) fn remote_peerid(&self) -> &Id {
        unwrap!(self.remote_peerid)
    }

    pub(crate) fn remote_addr(&self) -> &SocketAddr {
        unwrap!(self.remote_addr)
    }

    pub(crate) fn remote_ip(&self) -> IpAddr {
        unwrap!(self.remote_addr).ip()
    }

    pub(crate) fn remote_port(&self) -> u16 {
        unwrap!(self.remote_addr).port()
    }

    pub(crate) fn remote_name(&self) -> &str {
        unwrap!(self.remote_name).as_str()
    }

    pub(crate) fn remote_peer(&self) -> &PeerInfo {
        unwrap!(self.remote_peer)
    }

    pub(crate) fn upstream_addr(&self) -> &SocketAddr {
        unwrap!(self.upstream_addr)
    }

    pub(crate) fn upstream_name(&self) -> &str {
        unwrap!(self.upstream_name).as_str()
    }

    pub(crate) fn max_connections(&self) -> usize {
        self.max_connections
    }

    pub(crate) fn peer_keypair(&self) -> Option<&signature::KeyPair> {
        self.peer_keypair.as_ref()
    }

    pub(crate) fn upstream_domain(&self) -> Option<&str> {
        match self.domain_enabled {
            true => self.peer_domain.as_ref().map(|v|v.as_str()),
            false => None,
        }
    }

    pub(crate) fn rcvbuf(&self) -> Rc<RefCell<Box<Vec<u8>>>> {
        self.rcvbuf.clone()
    }

    pub(crate) fn upstream_peer(&self) -> Option<&PeerInfo> {
        self.peer.as_ref()
    }
}

#[macro_export]
macro_rules! srv_addr {
    ($inners:expr) => {{
        $inners.borrow().remote_addr()
    }};
}

#[macro_export]
macro_rules! srv_endp {
    ($inners:expr) => {{
        $inners.borrow().remote_name()
    }};
}

#[macro_export]
macro_rules! ups_endp {
    ($inners:expr) => {{
        $inners.borrow().upstream_name()
    }};
}
