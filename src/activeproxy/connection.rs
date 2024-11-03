
use std::mem;
use std::fmt;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::net::{
    SocketAddr,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr
};
use tokio::net::{TcpSocket, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{warn, error,info, debug, trace};

use crate::{
    srv_endp,
    ups_endp,
    unwrap,
    random_bytes,
    id, Id,
    Node,
    Error,
    error::Result,
    cryptobox, CryptoBox,
    Signature
};

use crate::activeproxy::{
    inners::InnerFields,
    packet::{Packet, AttachType, AuthType, ConnType, DisconnType},
    state::State,
};

// packet size (2bytes) + packet type(1bytes)
const PACKET_HEADER_BYTES: usize = mem::size_of::<u16>() + mem::size_of::<u8>();

static mut NEXT_CONNID: i32 = 0;
fn next_connection_id() -> i32 {
    unsafe {
    NEXT_CONNID += 1;
        if NEXT_CONNID == 0 {
        NEXT_CONNID += 1;
        }
        NEXT_CONNID
    }
}

#[allow(dead_code)]
pub(crate) struct ProxyConnection {
    node:           Arc<Mutex<Node>>,
    id:             i32,
    state:          State,
    keep_alive:     SystemTime,

    inners:         Rc<RefCell<InnerFields>>,

    disconnect_confirms: i32,       // TODO: volatile.

    relay:          Option<TcpStream>,
    upstream:       Option<TcpStream>,
    stickybuf:      Option<Vec<u8>>,

    //proxy:          Rc<RefCell<ProxyWorker>>,

    nonce:          Option<cryptobox::Nonce>,

    authorized_cb:  Box<dyn Fn(&ProxyConnection, &cryptobox::PublicKey, u16, bool)>,
    opend_cb:       Box<dyn Fn(&ProxyConnection)>,
    open_failed_cb: Box<dyn Fn(&ProxyConnection)>,
    closed_cb:      Box<dyn Fn(&ProxyConnection)>,
    busy_cb:        Box<dyn Fn(&ProxyConnection)>,
    idle_cb:        Box<dyn Fn(&ProxyConnection)>

}

#[allow(dead_code)]
impl ProxyConnection {
    pub(crate) fn new(node: Arc<Mutex<Node>>, inners: Rc<RefCell<InnerFields>>) -> Self {
        Self {
            node,
            id:             next_connection_id(),
            state:          State::Initializing,
            keep_alive:     SystemTime::now(),

            inners,

            disconnect_confirms: 0,

            relay:          None,
            upstream:       None,

            stickybuf:      Some(Vec::with_capacity(4*1024)),
            //proxy,
            nonce:          Some(cryptobox::Nonce::random()),

            authorized_cb:  Box::new(|_,_,_,_|{}),
            opend_cb:       Box::new(|_|{}),
            open_failed_cb: Box::new(|_|{}),
            closed_cb:      Box::new(|_|{}),
            busy_cb:        Box::new(|_|{}),
            idle_cb:        Box::new(|_|{}),
        }
    }

    pub(crate) fn inners(&self) -> Rc<RefCell<InnerFields>> {
        self.inners.clone()
    }

    pub(crate) fn node(&self) -> Arc<Mutex<Node>> {
        self.node.clone()
    }

    pub(crate) fn nodeid(&self) -> Id {
        self.node.lock().unwrap().id().clone()
    }

    pub(crate) fn id(&self) -> i32 {
        self.id
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn relay_mut(&mut self) -> &mut TcpStream {
        self.relay.as_mut().unwrap()
    }

    fn upstream_mut(&mut self) -> &mut TcpStream {
        self.upstream.as_mut().unwrap()
    }

    fn binding_socket(&self) -> TcpSocket {
        TcpSocket::new_v4().unwrap()
    }

    pub(crate) fn with_on_authorized_cb(&mut self, cb: Box<dyn Fn(&ProxyConnection, &cryptobox::PublicKey, u16, bool)>) {
        self.authorized_cb = cb;
    }

    pub(crate) fn with_on_opened_cb(&mut self, cb: Box<dyn Fn(&ProxyConnection)>) {
        self.opend_cb = cb;
    }

    pub(crate) fn with_on_open_failed_cb(&mut self, cb: Box<dyn Fn(&ProxyConnection)>) {
        self.open_failed_cb = cb;
    }

    pub(crate) fn with_on_closed_cb(&mut self, cb: Box<dyn Fn(&ProxyConnection)>) {
        self.closed_cb = cb;
    }

    pub(crate) fn with_on_busy_cb(&mut self, cb: Box<dyn Fn(&ProxyConnection)>) {
        self.busy_cb = cb;
    }

    pub(crate) fn with_on_idle_cb(&mut self, cb: Box<dyn Fn(&ProxyConnection)>) {
        self.idle_cb = cb;
    }

    fn stickybuf_mut(&mut self) -> &mut Vec<u8> {
        self.stickybuf.as_mut().unwrap()
    }

    fn stickybuf(&self) -> &[u8] {
        self.stickybuf.as_ref().unwrap()
    }

    fn encrypt_with_node(&self, plain: &[u8], cipher: &mut [u8]) -> Result<()> {
        self.node.lock().unwrap().encrypt(
            self.inners.borrow().remote_nodeid(),
            plain,
            cipher
        ).map(|_|())
    }

    fn decrypt_with_node(&self, cipher: &[u8], plain: &mut [u8]) -> Result<()> {
        self.node.lock().unwrap().decrypt(
            self.inners.borrow().remote_nodeid(),
            cipher,
            plain
        ).map_err(|e| {
            panic!(">>>> {e}");
        }).map(|_|())
    }

    fn sign_into_with_node(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.node.lock().unwrap().sign_into(data)
    }

    fn allow(&self, _: &SocketAddr) -> bool {
        true
    }

    fn on_authorized(&mut self, pk: &cryptobox::PublicKey, port: u16, domain_enabled: bool) {
        (self.authorized_cb)(self, pk, port, domain_enabled);
    }

    fn on_opened(&mut self) {
        (self.opend_cb)(self);
    }

    pub(crate) fn on_closed(&mut self) {
        (self.closed_cb)(self);
    }

    fn on_busy(&mut self) {
        (self.busy_cb)(self);
    }

    fn on_idle(&mut self) {
        (self.idle_cb)(self);
    }

    pub(crate) async fn close(&mut self) -> Result<()> {
        // unimplemented!()
        Ok(())
    }

    async fn close_upstream2(&self) -> Result<()> {
        unimplemented!()
    }

    async fn open_upstream(&mut self) -> Result<()> {
        debug!("Connection {} connecting to the upstream {}...", self.id, ups_endp!(self.inners));

        let ups_addr = self.inners.borrow().upstream_addr().clone();
        match self.binding_socket().connect(ups_addr).await {
            Ok(stream) => {
                info!("Connection {} has connected to server {}", self.id, srv_endp!(self.inners));
                self.upstream = Some(stream);
            },
            Err(e) => {
                error!("Connection {} connect to upstream {} failed: {}", self.id, ups_endp!(self.inners), e);
                _ = self.close_upstream2().await;
                self.state = State::Idling;
                self.on_idle();
            }
        };

        self.send_connect_response(self.upstream.is_some()).await
    }

    async fn close_upstream(&mut self) -> Result<()> {
        unimplemented!()
    }

    pub(crate) fn periodic_check(&mut self) {
        // unimplemented!()
    }

    pub(crate) async fn try_connect_server(&mut self) -> Result<()> {
        info!("Connection {} is connecting to the server {}...", self.id, srv_endp!(self.inners));

        let srv_addr = self.inners.borrow().remote_addr().clone();
        match self.binding_socket().connect(srv_addr).await {
            Ok(stream) => {
                info!("Connection {} has connected to server {}", self.id, srv_endp!(self.inners));
                self.relay = Some(stream);
                self.establish().await
            },
            Err(e) => {
                error!("Connection {} connect to server {} failed: {}", self.id, srv_endp!(self.inners), e);
                _ = self.close().await;
                Err(Error::from(e))
            }
        }
    }

    async fn establish(&mut self) -> Result<()>  {
        trace!("Connection {} started reading from the server.", self.id);

        let rcvbuf = self.inners.borrow().rcvbuf();
        let mut borrowed = rcvbuf.borrow_mut();
        match self.relay_mut().read(&mut borrowed.as_mut()[..]).await {
            Ok(n) if n == 0 => {
                info!("Connection {} was closed by the server.", self.id);
                Err(Error::State(format!("Connection {} was closed by the server.", self.id)))
            },
            Ok(len) => {
                self.on_relay_data(&borrowed[..len]).await
            },
            Err(e) => {
                error!("Connection {} failed to read server with error: {}", self.id, e);
                _ = self.close().await;
                Err(Error::from(e))
            }
        }
    }

    pub(crate) async fn on_relay_data(&mut self, input: &[u8]) -> Result<()> {
        self.keep_alive = SystemTime::now();

        let mut pos = 0;
        let mut remain = input.len();
        if self.stickybuf_mut().len() > 0 {
            if self.stickybuf().len() < PACKET_HEADER_BYTES {
                let rs = PACKET_HEADER_BYTES - self.stickybuf().len();
                //  Read header data, but insufficient to form a complete header
                if remain < rs {
                    self.stickybuf_mut().extend_from_slice(input);
                    return Ok(());
                }

                // A complete packet header has been read.
                self.stickybuf_mut().extend_from_slice(&input[..rs]);
                pos += rs;
                remain -= rs;
            }

            // Parse the header to determine packet size.
            let packet_sz = u16::from_be_bytes(self.stickybuf()[..1].try_into().unwrap()) as usize;
            let rs = packet_sz - self.stickybuf().len();
            if remain < rs {
                // Reader packet data but insufficient to form a complete packet
                self.stickybuf_mut().extend_from_slice(&input[pos..pos+remain]);
                return Ok(());
            }

            // A complete packet has been successfully read.
            self.stickybuf_mut().extend_from_slice(&input[pos..pos+rs]);
            pos += rs;
            remain -= rs;

            let mut stickybuf = self.stickybuf.take().unwrap();
            if let Err(_) = self.process_relay_packet(&stickybuf).await {
                return self.close().await;
            }
            stickybuf.clear();
            self.stickybuf = Some(stickybuf);
        }

        // Continue parsing the remaining data from input buffer.
        while remain > 0 {
            // clean sticky buffer to prepare for new packet.
            if remain < PACKET_HEADER_BYTES {
                self.stickybuf_mut().extend_from_slice(&input[pos..pos + remain]);
                return Ok(())
            }

            let packet_sz = u16::from_be_bytes(input[..size_of::<u16>()].try_into().unwrap()) as usize;
            if remain < packet_sz {
                // Reader packet data but insufficient to form a complete packet
                self.stickybuf_mut().extend_from_slice(&input[pos..pos+remain]);
                return Ok(())
            }

            if let Err(_) = self.process_relay_packet(&input[pos..pos+packet_sz]).await {
                return self.close().await;
            }
            pos += packet_sz;
            remain -= packet_sz;
        }
        Ok(())
    }

    async fn process_relay_packet(&mut self, input: &[u8]) -> Result<()> {
        let pos = mem::size_of::<u16>();
        if self.state == State::Initializing {
            return self.on_challenge(&input[pos..]).await;
        }

        // packet format
        // - u16: packet size,
        // - u8: packet flag.
        let result = Packet::from(input[pos]);
        if let Err(e) = result {
            error!("Received an invalid packet type: {}", e);
            return Err(e);
        }

        let packet = result.unwrap();
        debug!("Connection {} got packet from server {}: type={}, ack={}, size={}",
            self.id, srv_endp!(self.inners), packet, packet.ack(), input.len());

        if matches!(packet, Packet::Error(_)) {
            let len = input.len() - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;
            let mut plain = vec![0u8; len];
            _ = self.inners.borrow().cryptobox().encrypt(
                &input[PACKET_HEADER_BYTES..],
                &mut plain[..],
                self.nonce.as_ref().unwrap()
            )?;

            let mut pos = 0;
            let end = mem::size_of::<u16>();
            let ecode = u16::from_be_bytes(plain[pos..end].try_into().unwrap());

            pos = end;
            let data = &plain[pos..];
            let errstr = str::from_utf8(data).unwrap().to_string();

            error!("Connection {} got ERR response from the server {}, error:{}:{}",
                self.id, srv_endp!(self.inners), ecode, errstr);

            return Err(Error::Protocol(format!("Packet error")));
        }

        if !self.state.accept(&packet) {
            error!("Connection {} is not allowed for {} packet at {} state", self.id, packet, self.state);
            return Err(Error::Permission(format!("Permission denied")));
        }

        match packet {
            Packet::AuthAck(_)      => self.on_authenticate_response(input).await,
            Packet::AttachAck(_)    => self.on_attach_reponse(input).await,
            Packet::PingAck(_)      => self.on_ping_response(input).await,
            Packet::Connect(_)      => self.on_connect_request(input).await,
            Packet::Data(_)         => self.on_data_request(input).await,
            Packet::Disconnect(_)   => self.on_disconnect_request(input).await,
            Packet::DisconnectAck(_)=> self.on_disconnect_response(input).await,
            _ => {
                error!("INTERNAL ERROR: Connection {} got wrong {} packet in {} state", self.id, packet, self.state);
                Err(Error::Protocol(format!("Wrong expected packet {} received", packet)))
            }
        }
    }

    /*
    * Challenge packet
    * - plain
    *   - Random challenge bytes.
    */
    async fn on_challenge(&mut self, input: &[u8]) -> Result<()> {
        if input.len() < 32 || input.len() > 256 {
            error!("Connection {} got challenge from the server {}, size is error!",
                self.id, srv_endp!(self.inners));
            return Ok(())
        }
        // Sign the challenge, send auth or attach with siguature
        let sig = self.sign_into_with_node(input)?;
        if self.inners.borrow().is_authenticated() {
            self.send_attach_request(&sig).await
        } else {
            self.send_authenticate_request(&sig).await
        }
    }

    /*
    * AUTHACK packet payload:
    * - encrypted
    *   - sessionPk[server]
    *   - port[uint16]
    *   - domainEnabled[uint8]
    */
    const AUTH_ACK_SIZE: usize = PACKET_HEADER_BYTES    // header.
        + cryptobox::CryptoBox::MAC_BYTES               // MAC BYTES.
        + cryptobox::PublicKey::BYTES                   // public key.
        + mem::size_of::<u16>()                         // port.
        + mem::size_of::<u16>()                         // max connections allowed.
        + mem::size_of::<bool>();

    async fn on_authenticate_response(&mut self, input: &[u8]) -> Result<()> {
        if input.len() < Self::AUTH_ACK_SIZE {
            error!("Connection {} got an invalid AUTH ACK from server {}", self.id, srv_endp!(self.inners));
            return self.close().await;
        }

        debug!("Connection {} got AUTH ACK from server {}", self.id, srv_endp!(self.inners));


        let plain_len = Self::AUTH_ACK_SIZE - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;
        let mut plain = vec![0u8; plain_len];

        _ = self.decrypt_with_node(
            &input[PACKET_HEADER_BYTES..Self::AUTH_ACK_SIZE],
            &mut plain[..]
        )?;

        let mut pos = 0;
        let mut end = pos + cryptobox::PublicKey::BYTES;
        let server_pk = cryptobox::PublicKey::try_from( // extract server public key.
            &plain[pos..end]
        )?;

        pos = end;
        end += mem::size_of::<u16>();
        let port = u16::from_be_bytes(                  // extract port.
            plain[pos..end].try_into().unwrap()
        );

        pos = end;
        end += mem::size_of::<u16>();
        let max_connections = u16::from_be_bytes(       // extract max connections allowed
            plain[pos..end].try_into().unwrap()
        ) as usize;

        self.inners.borrow_mut().set_max_connections(max_connections);

        pos = end;
        let domain_enabled = input[pos] != 0;           // extract flag whether domain enabled or not.

        self.on_authorized(&server_pk, port, domain_enabled);

        self.state = State::Idling;
        self.on_opened();
        info!("Connection {} opened.", self.id);

        Ok(())
    }

    /*
     * No Payload.
     */
    async fn on_attach_reponse(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got ATTACH ACK from server {}", self.id, srv_endp!(self.inners));
        self.state = State::Idling;
        self.on_opened();
        Ok(())
    }

    /*
     * No Payload.
     */
    async fn on_ping_response(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got PING ACK from server {}", self.id, srv_endp!(self.inners));
        unimplemented!()
    }

    const CONNECT_REQ_SIZE: usize = PACKET_HEADER_BYTES
        + CryptoBox::MAC_BYTES
        + mem::size_of::<u8>()
        + 16
        + mem::size_of::<u16>();

    /*
     * CONNECT packet payload:
     * - encrypted
     *   - addrlen[uint8]
     *   - addr[16 bytes both for IPv4 or IPv6]
     *   - port[uint16]
     */
    async fn on_connect_request(&mut self, input: &[u8]) -> Result<()> {
        if input.len() < Self::CONNECT_REQ_SIZE {
            error!("Connection {} got an invalid CONNECT from server {}.", self.id, srv_endp!(self.inners));
            return Err(Error::Protocol(format!("Invalid CONNECT packet")));
        }

        debug!("Connection {} got CONNECT from server {}", self.id, srv_endp!(self.inners));
        self.state = State::Relaying;
        self.on_busy();

        let plain_len = Self::CONNECT_REQ_SIZE - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;  // TODO:
        let mut plain = vec![0u8; plain_len];
        self.inners.borrow().cryptobox().decrypt(
            &input[PACKET_HEADER_BYTES..Self::CONNECT_REQ_SIZE],
            &mut plain[..],
            self.nonce.as_ref().unwrap()
        )?;

        let mut pos = 0;
        let addr_len = plain[pos] as usize;

        pos += mem::size_of::<u8>();
        let ip = match (addr_len * 8) as u32 {
            Ipv4Addr::BITS => {
                let bytes = input[pos..pos + addr_len].try_into().unwrap();
                let bits = u32::from_be_bytes(bytes);
                IpAddr::V4(Ipv4Addr::from(bits))
            },
            Ipv6Addr::BITS => {
                let bytes = input[pos..pos + addr_len].try_into().unwrap();
                let bits = u128::from_be_bytes(bytes);
                IpAddr::V6(Ipv6Addr::from(bits))
            },
            _ => return Err(Error::Protocol(format!("Unsupported address family."))),
        };

        pos += 16;      // the length of the buffer for address.
        let end = pos + mem::size_of::<u16>();
        let port = u16::from_be_bytes(input[pos..end].try_into().unwrap());
        let addr = SocketAddr::new(ip, port);

        if self.allow(&addr) {
            self.open_upstream().await
        } else {
            self.send_connect_response(false).await?;
            self.state = State::Idling;
            self.on_idle();
            Ok(())
        }
    }

    /*
     * DATA packet payload:
     * - encrypted
     *   - data
     */
    async fn on_data_request(&mut self, input: &[u8]) -> Result<()> {
        trace!("Connection {} got DATA({}) from server {}", self.id, input.len(), srv_endp!(self.inners));

        let plain_len = input.len() - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;
        let mut data = vec![0u8; plain_len];
        _ = self.inners.borrow().cryptobox().decrypt(
            &input[PACKET_HEADER_BYTES..],
            &mut data[..],
            self.nonce.as_ref().unwrap()
        )?;

        trace!("Connection {} sending {} bytes data to upstream {}", self.id, data.len(), srv_endp!(self.inners));

        if let Err(e) = self.upstream.as_mut().unwrap().write_all(&data).await {
            error!("Connection {} sent to upstream {} failed: {}",
                self.id, srv_endp!(self.inners), e);
            self.close_upstream().await?;
        }
        Ok(())
    }

    /*
     * No payload
     */
    async fn on_disconnect_request(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got DISCONNECT from server {}", self.id, srv_endp!(self.inners));

        self.close_upstream().await?;
        self.send_disconnect_response().await?;

        self.disconnect_confirms += 1;
        if self.disconnect_confirms == 2 {
            self.disconnect_confirms = 0;
            self.state = State::Idling;
            self.on_idle();
        }
        Ok(())
    }

    /*
    * No payload
    */
    async fn on_disconnect_response(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got DISCONNECT_ACK from server {}", self.id, srv_endp!(self.inners));

        if self.disconnect_confirms == 2 {
            self.disconnect_confirms = 0;
            self.state = State::Idling;
            self.on_idle();
        }
        Ok(())
    }

    /*
    * ATTACH packet:
    *   - plain
    *     - clientNodeId
    *   - encrypted
    *     - sessionPk[client]
    *     - connectionNonce
    *     - signature[challenge]
    *   - plain
    *     - padding
    */
    async fn send_attach_request(&mut self, input: &[u8]) -> Result<()> {
        assert!(input.len() == Signature::BYTES);
        if self.state == State::Closed {
            return Ok(())
        }

        self.state = State::Attaching;

        let len = cryptobox::PublicKey::BYTES       // publickey
            + cryptobox::Nonce::BYTES               // nonce.
            + Signature::BYTES;                     // signature of challenge.
        let mut plain:Vec<u8> = Vec::with_capacity(len);

        plain.extend_from_slice(self.inners.borrow().session_keypair().public_key().as_bytes());
        plain.extend_from_slice(unwrap!(self.nonce).as_bytes());  // session nonce.
        plain.extend_from_slice(input);             // signature of challenge.

        let len = id::ID_BYTES                      // nodeid.
            + CryptoBox::MAC_BYTES                  // encryption MAC bytes.
            + plain.len();                          // data size.
        let mut payload: Vec<u8> = Vec::with_capacity(len);
        payload.extend_from_slice(self.node.lock().unwrap().id().as_bytes());
        payload.reserve(payload.len() + plain.len() + CryptoBox::MAC_BYTES);

        self.encrypt_with_node( // padding encrypted payload
            &plain,
            &mut payload[id::ID_BYTES..]
        )?;

        self.send_relay_packet(
            &Packet::Attach(AttachType),
            Some(&payload),
            |_|{}
        ).await
    }

    async fn send_authenticate_request(&mut self, input: &[u8]) -> Result<()> {
        debug_assert!(input.len() == Signature::BYTES);
        if self.state == State::Closed {
            return Ok(())
        }

        self.state = State::Authenticating;

        let domain_len = self.inners.borrow().upstream_domain().map_or(0, |v|v.len());
        let padding_sz = (random_padding() % 256) as usize;

        let len = cryptobox::PublicKey::BYTES   // session key.
            + cryptobox::Nonce::BYTES           // nonce.
            + Signature::BYTES                  // signature of challenge.
            + mem::size_of::<u8>()              // the value to domain length.
            + domain_len                        // domain string.
            + padding_sz;

        let mut plain = Vec::with_capacity(len);
        plain.extend_from_slice(self.inners.borrow().session_keypair().public_key().as_bytes());
        plain.extend_from_slice(unwrap!(self.nonce).as_bytes());
        plain.extend_from_slice(input);
        plain.extend_from_slice(&[domain_len as u8]);
        if domain_len > 0 {
            plain.extend_from_slice(
                unwrap!(self.inners.borrow().upstream_domain()).as_bytes()
            )
        }
        plain.extend_from_slice(&random_bytes(padding_sz));

        let len = id::ID_BYTES + CryptoBox::MAC_BYTES + plain.len();
        let mut payload =vec![0u8;len];
        payload[..id::ID_BYTES].copy_from_slice(self.node.lock().unwrap().id().as_bytes());
        self.encrypt_with_node( // padding encrypted payload.
            &plain,
            &mut payload[id::ID_BYTES..]
        )?;

        self.send_relay_packet(
            &Packet::Auth(AuthType),
            Some(&payload),
            |_|{},
        ).await
    }

    /*
     * CONNECTACK packet payload:
     * - plain
     *   - success[uint8]
     *   - padding
     */
    async fn send_connect_response(&mut self, is_success: bool) -> Result<()> {
        let data = random_boolean(is_success);
        self.send_relay_packet(
            &Packet::ConnectAck(ConnType),
            Some(&[data]),
            move |conn: &ProxyConnection| {
                if is_success {
                    _ = conn.start_read_upstream();
                }
            }
        ).await
    }

    /*
     * DISCONNECT packet:
     *   - plain
     *     - padding
     */
    async fn send_disconnect_response(&mut self) -> Result<()> {
        if self.state == State::Closed {
            return Ok(())
        }

        self.send_relay_packet(
            &Packet::Disconnect(DisconnType),
            None,
            |_|{},
        ).await
    }


    async fn send_relay_packet(&mut self,
        pkt: &Packet,
        input: Option<&[u8]>,
        cb: impl FnOnce(&ProxyConnection),
    ) -> Result<()> {
        if self.state == State::Closed {
            warn!("Connection {} is already closed, but still try to send {} to upstream.", self.id, pkt);
            return Ok(())
        }

        let mut sz: u16 = (PACKET_HEADER_BYTES + input.map_or(0, |v|v.len())) as u16;
        let mut padding_sz = 0;
        if !(matches!(pkt, Packet::Auth(_)) ||
            matches!(pkt, Packet::Data(_))  ||
            matches!(pkt, Packet::Error(_))) {

            padding_sz = random_padding() as usize;
            sz += padding_sz as u16
        }
        debug!("Connection {} send {} to server {}.", self.id, pkt, srv_endp!(self.inners));

        let len = PACKET_HEADER_BYTES               // packet header.
             + input.map_or(0, |v|v.len())          // packet payload.
             + padding_sz as usize;                 // padding size for randomness.


        let mut buf = Vec::with_capacity(len);
        buf.extend_from_slice(&sz.to_be_bytes());   // packet size.
        buf.extend_from_slice(&[pkt.value()]);      // packet flag.
        if let Some(payload) = input.as_ref() {
            buf.extend_from_slice(payload)          // packet payload
        }
        if padding_sz > 0 {                         // padding
            buf.extend_from_slice(&random_bytes(padding_sz))
        }

        match self.relay_mut().write_all(&mut buf).await {
            Ok(_) => cb(&self),
            Err(e) => {
                error!("Connection {} send {} to server {} failed: {}", self.id, pkt, srv_endp!(self.inners), e);
                self.close().await?;
            }
        }
        Ok(())
    }

     async fn start_read_upstream(&self) -> Result<()> {
        trace!("Connection {} start reading from the upstream.", self.id);

        /*
        let rcvbuf = self.inners.borrow().rcvbuf();
        let mut borrowed = rcvbuf.borrow_mut();
        match self.upstream_mut().read(&mut borrowed.as_mut()[..]).await {
            Ok(n) if n == 0 => {
                info!("Connection {} read upstream UV_EOF.", self->id);
                Err(Error::State(format!("Connection {} was closed by the server.", self.id)))
            },
            Ok(len) => {
                self.send_data_request(&borrowed[..len]).await
            },
            Err(e) => {
                error!("Connection {} read upstream error({}): {}.", self->id, e);
                _ = self.close().await;
                Err(Error::from(e))
            }
        }
        */
        unimplemented!()
     }
}

impl fmt::Display for ProxyConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection[{}]: state={}", self.id, self.state)?;
        Ok(())
    }
}

fn random_padding() -> u32 {
    unsafe {
        libsodium_sys::randombytes_random()
    }
}

fn random_boolean(input: bool) -> u8 {
    let val = unsafe {
        libsodium_sys::randombytes_random()
    } as u8;

    if input {
        val | 0x01
    } else {
        val & 0xFE
    }
}
