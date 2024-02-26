use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;
use tokio::io::{self};
use tokio::net::UdpSocket;
use tokio::runtime::{self};
use log::info;

use crate::{
    as_millis,
    version,
    dht::DHT,
    rpccall::RpcCall,
    node_runner::NodeRunner
};

use crate::msg::{
    msg::Msg
};

const RPC_SERVER_REACHABILITY_TIMEOUT: u128 = 60 * 1000;

#[derive(Clone, PartialEq, Eq)]
enum State {
    Initial,
    Running,
    Stopped
}

pub(crate) struct RpcServer {
    node_runner: Option<Rc<RefCell<NodeRunner>>>,

    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,

    state: State,
    started: SystemTime,

    reachable: bool,
    received_msgs: i32,
    msgs_atleast_reachable_check: i32,
    last_reachable_check: SystemTime,

    // next_txid: i32, // TODO atomic ?
}

#[allow(dead_code)]
impl RpcServer {
    pub(crate) fn new() -> Self {
        RpcServer {
            node_runner: None,
            dht4: None,
            dht6: None,
            state: State::Initial,
            started: SystemTime::UNIX_EPOCH,
            reachable: false,
            received_msgs: 0,
            msgs_atleast_reachable_check: 0,
            last_reachable_check: SystemTime::UNIX_EPOCH,

            // next_txid: 0,
        }
    }

    pub(crate) fn attach(&mut self, node_runner: Rc<RefCell<NodeRunner>>) {
        self.node_runner = Some(node_runner);
    }

    pub(crate) fn detach(&mut self) {
        _ = self.node_runner.take()
    }

    fn node_runner(&self) -> &Rc<RefCell<NodeRunner>> {
        self.node_runner.as_ref().unwrap()
    }

    pub(crate) fn enable_dht4(&mut self, dht4: Rc<RefCell<DHT>>) {
        self.dht4 = Some(dht4)
    }

    pub(crate) fn disable_dht4(&mut self) {
        _ = self.dht4.take()
    }

    fn has_dht4(&self) -> bool {
        self.dht4.is_some()
    }

    pub(crate) fn enable_dht6(&mut self, dht6: Rc<RefCell<DHT>>) {
        self.dht6 = Some(dht6)
    }

    pub(crate) fn disable_dht6(&mut self) {
        _ = self.dht6.take()
    }

    fn has_dht6(&self) -> bool {
        self.dht6.is_some()
    }

    pub(crate) fn is_reachable(&self) -> bool {
        self.reachable
    }

    pub(crate) fn update_reachability(&mut self) {
        // don't do pings too often if we're not receiving anything
        // (connection might be dead)

        if self.received_msgs != self.msgs_atleast_reachable_check {
            self.reachable = true;
            self.last_reachable_check = SystemTime::now();
            self.msgs_atleast_reachable_check = self.received_msgs;
            return;
        }

        if as_millis!(self.last_reachable_check) > RPC_SERVER_REACHABILITY_TIMEOUT {
            self.reachable = false;
        }
    }

    pub(crate) fn start(&mut self) {
        if self.state != State::Initial {
            return;
        }

        // open sockets

        self.state = State::Running;
        self.started = SystemTime::now();

        if let Some(dht4) = self.dht4.as_ref() {
            info!("Started RPC server on ipv4 address: {}", dht4.borrow().addr());
        }
        if let Some(dht6) = self.dht6.as_ref() {
            info!("Started RPC server on ipv6 address: {}", dht6.borrow().addr());
        }
    }

    fn stop(&mut self) {
        if self.state == State::Stopped {
            return;
        }

        self.state = State::Stopped;

        // TODO
        if let Some(dht4) = self.dht4.as_ref() {
            info!("Stopped RPC server on ipv4: {}", dht4.borrow().addr());
        }
        if let Some(dht6) = self.dht6.as_ref() {
            info!("Started RPC server on ipv6: {}", dht6.borrow().addr());
        }
    }

    fn run_loop(&self) -> io::Result<()> {
        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(async move {
            let mut sock4: Option<UdpSocket> = None;
            if let Some(dht4) = self.dht4.as_ref(){
                sock4 = Some(UdpSocket::bind(dht4.borrow().addr()).await?);
            }
            let mut sock6: Option<UdpSocket> = None;
            if let Some(dht6) = self.dht6.as_ref(){
                sock6 = Some(UdpSocket::bind(dht6.borrow().addr()).await?);
            }

            loop {
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

                    // Writable event for either socket
                    _ = write_socket(sock4.as_ref()) => {
                        println!("Socket1 is writable");
                    }

                    _ = write_socket(sock6.as_ref()) => {
                        println!("Socket2 is writable");
                    }
                }
            }
        })
    }

    pub(crate) fn send_msg(&self, mut msg: Box<dyn Msg>) {
        msg.with_ver(version::build(
            version::NODE_SHORT_NAME,
            version::NODE_VERSION
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

async fn read_socket(socket_option: Option<&UdpSocket>) -> Result<Vec<u8>, std::io::Error> {
    match socket_option.as_ref() {
        Some(socket) => {
            let mut buffer = vec![0; 1024];
            let (size, _) = socket.recv_from(&mut buffer).await?;
            buffer.truncate(size);
            Ok(buffer)
        },
        None => {
            Err(io::Error::new(io::ErrorKind::NotFound, "unavailable"))
        }
    }
}

async fn write_socket(socket_option: Option<&UdpSocket>) -> Result<(), std::io::Error> {
    match socket_option.as_ref() {
        Some(socket) => {
            let message = b"Hello, World!";
            socket.send(message).await?;
            Ok(())
        },
        None => {
            Err(io::Error::new(io::ErrorKind::NotFound, "unavailable"))
        }
    }
}
