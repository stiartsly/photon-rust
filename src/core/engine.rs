use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;
use std::time::SystemTime;

use log::info;
use tokio::io;
use tokio::net::UdpSocket;
use tokio::runtime;

use crate::{
    data_storage::DataStorage, dht::DHT, error::Error, id::Id, lookup_option::LookupOption,
    node::Node, rpccall::RpcCall, sqlite_storage::SqliteStorage, token_man::TokenManager, version,
};

use crate::msg::msg::Msg;

#[allow(dead_code)]
pub(crate) struct NodeEngine {
    id: Id,
    storage_path: String,

    running: bool,
    started: SystemTime,

    reachable: bool,
    received_msgs: i32,
    msgs_atleast_reachable_check: i32,
    last_reachable_check: SystemTime,

    pub(crate) dht4: Option<Rc<RefCell<DHT>>>,
    pub(crate) dht6: Option<Rc<RefCell<DHT>>>,
    dht_num: i32,
    option: LookupOption,

    pub(crate) token_man: Rc<RefCell<TokenManager>>,
    pub(crate) storage: Rc<RefCell<dyn DataStorage>>,
    // encryption_ctxts: CryptoCache,
}

#[allow(dead_code)]
impl NodeEngine {
    pub fn new(id: Id, storage_path: &str) -> Result<Self, Error> {
        Ok(NodeEngine {
            id: id.clone(),

            storage_path: storage_path.to_string(),
            started: SystemTime::UNIX_EPOCH,
            running: false,

            reachable: false,
            received_msgs: 0,
            msgs_atleast_reachable_check: 0,
            last_reachable_check: SystemTime::UNIX_EPOCH,

            dht4: None,
            dht6: None,
            dht_num: 0,
            option: LookupOption::Conservative,

            token_man: Rc::new(RefCell::new(TokenManager::new())),
            storage: Rc::new(RefCell::new(SqliteStorage::new())),
        })
    }

    pub fn start(&mut self, dht4: Option<Rc<RefCell<DHT>>>, dht6: Option<Rc<RefCell<DHT>>>) {
        if let Some(dht) = dht4 {
            dht.borrow_mut()
                .enable_persistence(&format!("{}/dht4.cache", self.storage_path));
            self.dht4 = Some(Rc::clone(&dht));
            info!(
                "Started RPC server on ipv4 address: {}",
                dht.borrow().addr()
            );
        }

        if let Some(dht) = dht6 {
            dht.borrow_mut()
                .enable_persistence(&format!("{}/dht6.cache", self.storage_path));
            self.dht6 = Some(Rc::clone(&dht));
            info!(
                "Started RPC server on ipv6 address: {}",
                dht.borrow().addr()
            );
        }

        let dbpath = self.storage_path.clone() + "/node.db";
        self.storage.borrow_mut().open(dbpath.as_str());
    }

    pub(crate) fn run_loop(&mut self) -> io::Result<()> {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let mut sock4: Option<UdpSocket> = None;
            if let Some(dht4) = self.dht4.as_ref() {
                sock4 = Some(UdpSocket::bind(dht4.borrow().addr()).await?);
            }
            let mut sock6: Option<UdpSocket> = None;
            if let Some(dht6) = self.dht6.as_ref() {
                sock6 = Some(UdpSocket::bind(dht6.borrow().addr()).await?);
            }

            while self.running {
                tokio::select! {
                    rc1 = read_socket(sock4.as_ref()) => {
                        match rc1 {
                            Ok(data) => println!("Received data on socket1: {:?}", data),
                            Err(err) => eprintln!("Error reading from socket1: {}", err),
                        }
                    }

                    rc2 = read_socket(sock6.as_ref()) => {
                        match rc2 {
                            Ok(data) => println!("Received data on socket2: {:?}", data),
                            Err(err) => eprintln!("Error reading from socket2: {}", err),
                        }
                    }

                    rc3 = write_socket(sock4.as_ref()) => {
                        match rc3 {
                           Ok(_) => println!("Written data on socket1 "),
                           Err(err) => eprintln!("Error writing to socket1 {}", err),
                        }
                    }

                    rc4 = write_socket(sock6.as_ref()) => {
                        match rc4 {
                           Ok(_) => println!("Written data on socket2 "),
                           Err(err) => eprintln!("Error writing to socket2 {}", err),
                        }
                    }
                }
                self.running = false;
            }

            Ok(())
        })
    }

    pub fn stop(&mut self) {
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

    pub(crate) fn storage(&self) -> Rc<RefCell<dyn DataStorage>> {
        unimplemented!()
    }

    pub async fn bootstrap(&self, _: &[Node]) -> Result<(), Error> {
        unimplemented!()
    }

    pub(crate) fn is_reachable(&self) -> bool {
        self.reachable
    }

    fn persistent_announce(&mut self) {
        info!("Reannounce the perisitent values and peers...");

        // let mut timestamp = SystemTime::now();
        unimplemented!()
    }

    pub(crate) fn send_msg(&self, mut msg: Box<dyn Msg>) {
        msg.with_ver(version::build(
            version::NODE_SHORT_NAME,
            version::NODE_VERSION,
        ));

        if let Some(mut call) = msg.associated_call() {
            call.dht().borrow().on_send(call.target_id());
            call.send(&self);
        }

        //sendData(msg);
    }

    pub(crate) fn send_call(&self, _: Box<RpcCall>) {
        unimplemented!()
    }
}

pub(crate) fn start_tweak(
    engine: &Rc<RefCell<NodeEngine>>,
    addr4: Option<SocketAddr>,
    addr6: Option<SocketAddr>,
) {
    let mut dht4: Option<Rc<RefCell<DHT>>> = None;
    let mut dht6: Option<Rc<RefCell<DHT>>> = None;

    if let Some(addr) = addr4 {
        dht4 = Some(Rc::new(RefCell::new(DHT::new(&engine, &addr))));
    }
    if let Some(addr) = addr6 {
        dht6 = Some(Rc::new(RefCell::new(DHT::new(&engine, &addr))));
    }
    engine.borrow_mut().start(dht4, dht6);
}

pub(crate) fn stop_tweak(engine: &Rc<RefCell<NodeEngine>>) {
    engine.borrow_mut().stop()
}

async fn read_socket(socket_option: Option<&UdpSocket>) -> Result<Vec<u8>, std::io::Error> {
    match socket_option.as_ref() {
        Some(socket) => {
            let mut buffer = vec![0; 1024];
            let (size, _) = socket.recv_from(&mut buffer).await?;
            buffer.truncate(size);
            Ok(buffer)
        },
        None => {
            // Err(io::Error::new(io::ErrorKind::NotFound, "unavailable"))
            use std::time::Duration;
            use tokio::time::sleep;
            sleep(Duration::from_millis(68719476734)).await;
            Ok(Vec::new())
        }
    }
}

async fn write_socket(socket_option: Option<&UdpSocket>) -> Result<(), std::io::Error> {
    match socket_option.as_ref() {
        Some(socket) => {
            use std::time::Duration;
            use tokio::time::sleep;
            sleep(Duration::from_millis(5000)).await;
            let message = b"Hello, World!";
            let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
            socket.send_to(message, target).await?;
            Ok(())
        },
        None => {
            // Do nothing when socket is None
            //Err(io::Error::new(io::ErrorKind::NotFound, "unavailable"))
            use std::time::Duration;
            use tokio::time::sleep;
            sleep(Duration::from_millis(68719476734)).await;
            Ok(())
        }
    }
}
