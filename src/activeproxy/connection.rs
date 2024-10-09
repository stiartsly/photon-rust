
use std::mem;
use std::fmt;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
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
    random_bytes,
    id,
    Error,
    error::Result,
    cryptobox, CryptoBox,
    Signature
};

use crate::activeproxy::{
    proxy::ProxyClient,
    packet::{Packet, AttachType, AuthType, ConnType, DisconnType},
};

#[allow(dead_code)]
#[derive(PartialEq)]
pub(crate) enum State {
    Initializing = 0,
    Authenticating,
    Attaching,
    Idling,
    Relaying,
    Disconnecting,
    Closed
}

impl State {
    fn accept(&self, pkt: &Packet) -> bool {
        match self {
            State::Initializing     => false,
            State::Authenticating   => matches!(pkt, Packet::AuthAck(_)),
            State::Attaching        => matches!(pkt, Packet::AttachAck(_)),
            State::Idling           => matches!(pkt, Packet::PingAck(_)) ||
                                       matches!(pkt, Packet::Connect(_)),
            State::Relaying         => matches!(pkt, Packet::PingAck(_)) ||
                                       matches!(pkt, Packet::Data(_)) ||
                                       matches!(pkt, Packet::Disconnect(_)),
            State::Disconnecting    => matches!(pkt, Packet::Disconnect(_)) ||
                                       matches!(pkt, Packet::Data(_)) ||
                                       matches!(pkt, Packet::DisconnectAck(_)),
            State::Closed           => false,
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

macro_rules! srv_endp {
    ($client:expr) => {{
        $client.borrow().srv_endp().borrow().as_ref().unwrap()
    }};
}

macro_rules! ups_endp {
    ($client:expr) => {{
        $client.borrow().ups_endp()
    }};
}

// packet size + packet type.
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
    id: i32,

    state: State,
    keep_alive: SystemTime,

    disconnect_confirms: i32,  // TODO: volatile.

    relay: Option<TcpStream>,
    upstream: Option<TcpStream>,

    stickybuf: Option<Vec<u8>>,

    proxy: Rc<RefCell<ProxyClient>>,

    nonce: Option<cryptobox::Nonce>
}

#[allow(dead_code)]
impl ProxyConnection {
    pub(crate) fn new(proxy: Rc<RefCell<ProxyClient>>) -> Self {
        Self {
            id: next_connection_id(),
            state: State::Initializing,
            keep_alive: SystemTime::now(),

            disconnect_confirms: 0,

            relay   : None,
            upstream: None,

            stickybuf: Some(Vec::with_capacity(4*1024)),
            proxy,
            nonce   : None
        }
    }

    pub(crate) fn id(&self) -> i32 {
        self.id
    }

    fn relay_mut(&mut self) -> &mut TcpStream {
        self.relay.as_mut().unwrap()
    }

    fn upstream_mut(&mut self) -> &mut TcpStream {
        self.upstream.as_mut().unwrap()
    }

    fn binding_socket(&self) -> TcpSocket {
        unimplemented!()
    }

    fn proxy(&self) -> Rc<RefCell<ProxyClient>> {
        self.proxy.clone()
    }

    pub(crate) fn with_on_authorized_cb(&mut self, _cb: Box<dyn FnOnce(&ProxyConnection, &cryptobox::PublicKey, u16, bool)>) {
        unimplemented!()
    }

    pub(crate) fn with_on_opened_cb(&mut self, _: Box<dyn FnOnce(&ProxyConnection)>) {
        unimplemented!()
    }

    pub(crate) fn with_on_open_failed_cb(&mut self, _: Box<dyn FnOnce(&ProxyConnection)>) {
        unimplemented!()
    }

    pub(crate) fn with_on_closed_cb(&mut self, _: Box<dyn FnOnce(&ProxyConnection)>) {
        unimplemented!()
    }

    pub(crate) fn with_on_busy_cb(&mut self, _: Box<dyn FnOnce(&ProxyConnection)>) {
        unimplemented!()
    }

    pub(crate) fn with_on_idle_cb(&mut self, _: Box<dyn FnOnce(&ProxyConnection)>) {
        unimplemented!()
    }

    async fn on_authorized(&mut self, _: &cryptobox::PublicKey, _: u16, _: bool) {
        unimplemented!()
    }

    async fn on_opened(&mut self) -> Result<()> {
        unimplemented!()
    }

    pub(crate) async fn on_closed(&mut self) -> Result<()> {
        unimplemented!()
    }

    async fn on_busy(&mut self) -> Result<()> {
        unimplemented!()
    }

    async fn on_idle(&mut self) -> Result<()> {
        unimplemented!()
    }

    pub(crate) async fn close(&mut self) -> Result<()> {
        unimplemented!()
    }

    async fn open_upstream(&mut self) -> Result<()> {
        debug!("Connection {} connecting to the upstream {}...", self.id, ups_endp!(self.proxy));

        unimplemented!()
    }

    async fn close_upstream(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn stickybuf_mut(&mut self) -> &mut Vec<u8> {
        self.stickybuf.as_mut().unwrap()
    }

    fn stickybuf(&self) -> &[u8] {
        self.stickybuf.as_ref().unwrap()
    }

    pub(crate) fn periodic_check(&mut self) {
        unimplemented!()
    }

    pub(crate) async fn try_connect_server(&mut self) -> Result<()> {
        let cloned = self.proxy.clone();

        info!("Connection {} is connecting to the server {}...", self.id, srv_endp!(cloned));

        let srv_addr = cloned.borrow().srv_addr().clone();
        let srv_addr = srv_addr.borrow().unwrap();
        match self.binding_socket().connect(srv_addr).await {
            Ok(stream) => {
                info!("Connection {} has connected to server {}", self.id, srv_endp!(cloned));
                self.relay = Some(stream);
                Ok(())
            },
            Err(e) => {
                error!("Connection {} connect to server {} failed: {}", self.id, srv_endp!(cloned), e);
                _ = self.close().await;
                Err(Error::from(e))
            }
        }
    }

    async fn try_establish(&mut self) -> Result<()>  {
        trace!("Connection {} started reading from the server.", self.id);

        let proxy_cloned = self.proxy.clone();
        let mut borrowed = proxy_cloned.borrow_mut();
        let mut buf = borrowed.rcvbuf();

        match self.relay_mut().read(&mut buf).await {
            Ok(n) if n == 0 => {
                info!("Connection {} was closed by the server.", self.id);
                Err(Error::State(format!("Connection {} was closed by the server.", self.id)))
            },
            Ok(_) => {
                self.on_relay_data(&buf).await
            },
            Err(e) => {
                error!("Connection {} failed to read server with error: {}", self.id, e);
                _ = self.close().await;
                Err(Error::from(e))
            }
        }
    }

    async fn on_relay_data(&mut self, input: &[u8]) -> Result<()> {
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

            let stickybuf = self.stickybuf.take().unwrap();
            if let Err(_) = self.process_relay_packet(&stickybuf).await {
                return self.close().await;
            }
            self.stickybuf = Some(stickybuf);
        }

        // Continue parsing the remaining data from input buffer.
        while remain > 0 {
            // clean sticky buffer to prepare for new packet.
            self.stickybuf_mut().clear();

            if remain < PACKET_HEADER_BYTES {
                self.stickybuf_mut().extend_from_slice(&input[pos..pos + remain]);
                return Ok(())
            }
            let packet_sz = u16::from_be_bytes(self.stickybuf()[..1].try_into().unwrap()) as usize;
            if remain < packet_sz {
                // Reader packet data but insufficient to form a complete packet
                self.stickybuf_mut().extend_from_slice(&input[pos..pos+remain]);
                return Ok(())
            }

            let stickybuf = self.stickybuf.take().unwrap();
            if let Err(_) = self.process_relay_packet(&stickybuf).await {
                return self.close().await;
            }

            self.stickybuf = Some(stickybuf);

            // Continue parsing the data on input buffer.
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
        trace!("Connection {} got packet from server {}: type={}, ack={}, size={}",
            self.id, srv_endp!(self.proxy), packet.type_(), packet.ack(), input.len());

        if matches!(packet, Packet::Error(_)) {
            let len = input.len() - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;
            let mut plain = vec![0u8; len];
            _ = self.proxy.borrow().decrypt(
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
                self.id, srv_endp!(self.proxy), ecode, errstr);

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
                self.id, srv_endp!(self.proxy));
            return Ok(())
        }
        // Sign the challenge, send auth or attach with siguature
        let sig = self.proxy.borrow().sign_into_with_node(input)?;
        if self.proxy.borrow().is_authenticated() {
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
            error!("Connection {} got an invalid AUTH ACK from server {}", self.id, srv_endp!(self.proxy));
            return self.close().await;
        }

        debug!("Connection {} got AUTH ACK from server {}", self.id, srv_endp!(self.proxy));

        let plain_len = Self::AUTH_ACK_SIZE - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;
        let mut plain = vec![0u8; plain_len];

        _ = self.proxy.borrow().decrypt_with_node(
            &input[PACKET_HEADER_BYTES..],
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
        );
        self.proxy.borrow_mut().set_max_connections(max_connections as usize);

        pos = end;
        let domain_enabled = input[pos] != 0;           // extract flag whether domain enabled or not.

        self.on_authorized(&server_pk, port, domain_enabled).await;

        self.state = State::Idling;
        self.on_opened().await?;
        info!("Connection {} opened.", self.id);

        Ok(())
    }

    /*
     * No Payload.
     */
    async fn on_attach_reponse(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got ATTACH ACK from server {}", self.id, srv_endp!(self.proxy));
        self.state = State::Idling;
        self.on_opened().await
    }

    /*
     * No Payload.
     */
    async fn on_ping_response(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got PING ACK from server {}", self.id, srv_endp!(self.proxy));
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
            error!("Connection {} got an invalid CONNECT from server {}.", self.id, srv_endp!(self.proxy));
            return Err(Error::Protocol(format!("Invalid CONNECT packet")));
        }

        debug!("Connection {} got CONNECT from server {}", self.id, srv_endp!(self.proxy));
        self.state = State::Relaying;
        self.on_busy().await?;

        let plain_len = Self::CONNECT_REQ_SIZE - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;
        let mut plain = vec![0u8; plain_len];
        _ = self.proxy.borrow().decrypt(
            &input[PACKET_HEADER_BYTES..PACKET_HEADER_BYTES + Self::CONNECT_REQ_SIZE],
            &mut plain[..],
            self.nonce.as_ref().unwrap()
        )?;

        let mut pos = 0;
        let addr_len = plain[pos] as usize;

        pos += mem::size_of::<u8>();
        let ip = match addr_len as u32 {
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

        if self.proxy.borrow().allow(&addr) {
            self.open_upstream().await
        } else {
            self.send_connect_response(false).await?;
            self.state = State::Idling;
            self.on_idle().await
        }
    }

    /*
     * DATA packet payload:
     * - encrypted
     *   - data
     */
    async fn on_data_request(&mut self, input: &[u8]) -> Result<()> {
        trace!("Connection {} got DATA({}) from server {}", self.id, input.len(), srv_endp!(self.proxy));

        let plain_len = input.len() - PACKET_HEADER_BYTES - CryptoBox::MAC_BYTES;
        let mut data = vec![0u8; plain_len];
        _ = self.proxy.borrow().decrypt(
            &input[PACKET_HEADER_BYTES..],
            &mut data[..],
            self.nonce.as_ref().unwrap()
        )?;

        trace!("Connection {} sending {} bytes data to upstream {}", self.id, data.len(), ups_endp!(self.proxy));

        if let Err(e) = self.upstream.as_mut().unwrap().write_all(&data).await {
            error!("Connection {} sent to upstream {} failed: {}",
                self.id, ups_endp!(self.proxy), e);
            self.close_upstream().await?;
        }
        Ok(())
    }

    /*
     * No payload
     */
    async fn on_disconnect_request(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got DISCONNECT from server {}", self.id, srv_endp!(self.proxy));

        self.close_upstream().await?;
        self.send_disconnect_response().await?;

        self.disconnect_confirms += 1;
        if self.disconnect_confirms == 2 {
            self.disconnect_confirms = 0;
            self.state = State::Idling;
            self.on_idle().await?;
        }
        Ok(())
    }

    /*
    * No payload
    */
    async fn on_disconnect_response(&mut self, _input: &[u8]) -> Result<()> {
        debug!("Connection {} got DISCONNECT_ACK from server {}", self.id, srv_endp!(self.proxy));

        if self.disconnect_confirms == 2 {
            self.disconnect_confirms = 0;
            self.state = State::Idling;
            self.on_idle().await?;
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
        let nonce = cryptobox::Nonce::random();

        let len = cryptobox::PublicKey::BYTES       // publickey
            + cryptobox::Nonce::BYTES               // nonce.
            + Signature::BYTES;                     // signature of challenge.
        let mut plain:Vec<u8> = Vec::with_capacity(len);

        plain.extend_from_slice(self.proxy.borrow().session_keypair().borrow().public_key().as_bytes());
        plain.extend_from_slice(nonce.as_bytes());  // session nonce.
        plain.extend_from_slice(input);             // signature of challenge.

        let len = id::ID_BYTES                      // nodeid.
            + CryptoBox::MAC_BYTES                  // encryption MAC bytes.
            + plain.len();                          // data size.
        let mut payload: Vec<u8> = Vec::with_capacity(len);
        payload.extend_from_slice(self.proxy.borrow().nodeid().borrow().as_ref().unwrap().as_bytes());
        self.proxy.borrow_mut().encrypt_with_node( // padding encrypted payload
            &plain,
            &mut payload[id::ID_BYTES..]
        )?;

        self.send_relay_packet(
            &Packet::Attach(AttachType),
            Some(&payload),
            None
        ).await
    }

    async fn send_authenticate_request(&mut self, input: &[u8]) -> Result<()> {
        debug_assert!(input.len() == Signature::BYTES);
        if self.state == State::Closed {
            return Ok(())
        }

        self.state = State::Authenticating;

        let nonce = cryptobox::Nonce::random();
        let domain_len = self.proxy.borrow().domain_name().map_or(0, |v|v.len());
        let padding_sz = random_padding() as usize;

        let len = cryptobox::PublicKey::BYTES   // session key.
            + cryptobox::Nonce::BYTES           // nonce.
            + Signature::BYTES                  // signature of challenge.
            + mem::size_of::<u8>()              // the value to domain length.
            + domain_len                        // domain string.
            + padding_sz;

        let mut plain = Vec::with_capacity(len);
        plain.extend_from_slice(self.proxy.borrow().session_keypair().borrow().public_key().as_bytes());
        plain.extend_from_slice(nonce.as_bytes());
        plain.extend_from_slice(input);
        plain.extend_from_slice(&[domain_len as u8]);
        if domain_len > 0 {
            plain.extend_from_slice(
                self.proxy.borrow().domain_name().unwrap().as_bytes()
            )
        }
        plain.extend_from_slice(&random_bytes(padding_sz));

        let len = id::ID_BYTES + CryptoBox::MAC_BYTES + plain.len();
        let mut payload = Vec::with_capacity(len);
        payload.extend_from_slice(self.proxy.borrow().nodeid().borrow().as_ref().unwrap().as_bytes());

        self.proxy.borrow().encrypt_with_node( // padding encrypted payload.
            &plain,
            &mut payload[id::ID_BYTES..]
        )?;

        self.send_relay_packet(
            &Packet::Auth(AuthType),
            Some(&payload),
            None
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
        let cb = |_: &ProxyConnection| {
            //if is_success && conn.upstream.data {
            //    conn.start_read_upstream().await
            //}
        };
        self.send_relay_packet(
            &Packet::ConnectAck(ConnType),
            Some(&[data]),
            Some(Box::new(cb))
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
            None
        ).await
    }

    async fn send_relay_packet(&mut self,
        pkt: &Packet,
        input: Option<&[u8]>,
        cb: Option<Box<dyn FnOnce(&ProxyConnection)>>
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

        debug!("Connection {} send {} to server {}.", self.id, pkt, srv_endp!(self.proxy));

        let len = mem::size_of::<u16>()             // packet size.
             + mem::size_of::<u8>()                 // packet flag.
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
            Ok(_) => cb.map(|cb| cb(&self)).unwrap(),
            Err(e) => {
                error!("Connection {} send {} to server {} failed: {}", self.id, pkt, srv_endp!(self.proxy), e);
                self.close().await?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for ProxyConnection {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

fn random_padding() -> u32 {
    unsafe {
        libsodium_sys::randombytes_random()
    }
}

fn random_boolean(_: bool) -> u8 {
    unimplemented!()
}
