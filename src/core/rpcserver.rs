use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;
use tokio::io::{self, Interest};
use tokio::net::UdpSocket;
use tokio::runtime::{self};
use log::{info};

use crate::version;
use crate::dht::DHT;
use crate::rpccall::RpcCall;
use crate::msg::msg::Msg;

#[derive(Clone, PartialEq, Eq)]
enum State {
    Initial,
    Running,
    Stopped
}

pub(crate) struct RpcServer {
    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,

    state: State,
    started: SystemTime,

    reachable: bool
}

#[allow(dead_code)]
impl RpcServer {
    pub(crate) fn new() -> Self {
        RpcServer {
            dht4: None,
            dht6: None,
            state: State::Initial,
            started: SystemTime::UNIX_EPOCH,
            reachable: false,
        }
    }

    pub(crate) fn enable_dht4(&mut self, dht4: Rc<RefCell<DHT>>) {
        self.dht4 = Some(dht4)
    }

    pub(crate) fn disable_dht4(&mut self) {
        _ = self.dht4.take()
    }

    pub(crate) fn enable_dht6(&mut self, dht6: Rc<RefCell<DHT>>) {
        self.dht6 = Some(dht6)
    }

    pub(crate) fn disable_dht6(&mut self) {
        _ = self.dht6.take()
    }

    pub(crate) fn is_reachable(&self) -> bool {
        self.reachable
    }

    fn start(&mut self) {
        if self.state != State::Initial {
            return;
        }

        // open sockets

        self.state = State::Running;
        self.started = SystemTime::now();

        if let Some(dht4) = self.dht4.as_ref() {
            info!("Started RPC server on ipv4 address: {}", dht4.borrow().origin());
        }
        if let Some(dht6) = self.dht6.as_ref() {
            info!("Started RPC server on ipv6 address: {}", dht6.borrow().origin());
        }
    }

    fn stop(&mut self) {
        if self.state == State::Stopped {
            return;
        }

        self.state = State::Stopped;

        // TODO

        if let Some(dht4) = self.dht4.as_ref() {
            info!("Stopped RPC server on ipv4: {}", dht4.borrow().origin());
        }
        if let Some(dht6) = self.dht6.as_ref() {
            info!("Started RPC server on ipv6: {}", dht6.borrow().origin());
        }
    }

    fn run_loop(&self) -> io::Result<()> {
        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(async move {
            let mut socket4: Option<UdpSocket> = None;
            if let Some(dht4) = self.dht4.as_ref(){
                socket4 = Some(UdpSocket::bind(dht4.borrow().origin()).await?);
            }
            let mut socket6: Option<UdpSocket> = None;
            if let Some(dht6) = self.dht6.as_ref(){
                socket6 = Some(UdpSocket::bind(dht6.borrow().origin()).await?);
            }

            loop {
                if let Some(sock4) = socket4.as_ref() {
                    let ready = sock4.ready(Interest::READABLE | Interest::WRITABLE).await?;
                    if ready.is_readable() {
                        let mut data = [0; 1024];
                        match sock4.try_recv(&mut data[..]) {
                            Ok(n) => {
                                println!("received {:?}", &data[..n]);
                            }
                            // False-positive, continue
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    if ready.is_writable() {
                        match sock4.try_send(b"hello world") {
                            Ok(n) => {
                                println!("sent {} bytes", n);
                            }
                            // False-positive, continue
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                }

                if let Some(sock6) = socket6.as_ref() {
                    let ready = sock6.ready(Interest::READABLE | Interest::WRITABLE).await?;
                    if ready.is_readable() {
                        let mut data = [0; 1024];
                        match sock6.try_recv(&mut data[..]) {
                            Ok(n) => {
                                println!("received {:?}", &data[..n]);
                            }
                            // False-positive, continue
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    if ready.is_writable() {
                        match sock6.try_send(b"hello world") {
                            Ok(n) => {
                                println!("sent {} bytes", n);
                            }
                            // False-positive, continue
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
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

        if let Some(call) = msg.associated_call() {
            call.dht().borrow().on_send(call.target_id());
            call.send();
        }

        //sendData(msg);
    }

    pub(crate) fn send_call(&self, _: Box<RpcCall>) {
        unimplemented!()
    }
}


/*
use tokio::net::UdpSocket;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::time::Duration;
use tokio::runtime::Runtime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn send_receive(socket: UdpSocket, target_addr: SocketAddr, message: &'static str) {
    loop {
        // Sending data
        match socket.send_to(message.as_bytes(), target_addr).await {
            Ok(_) => println!("Sent message: {}", message),
            Err(e) => eprintln!("Error sending message: {}", e),
        }

        // Reading data
        let mut buffer = [0u8; 1024];
        match socket.recv_from(&mut buffer).await {
            Ok((size, src)) => {
                let received_message = String::from_utf8_lossy(&buffer[..size]);
                println!("Received message from {}: {}", src, received_message);
            }
            Err(e) => eprintln!("Error receiving message: {}", e),
        }

        // Sleep for a while before the next iteration
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn main() {
    // Create a single-threaded Tokio runtime
    let rt = Runtime::new_current_thread().unwrap();

    // Run the main asynchronous function
    rt.block_on(async {``
        // Binding UDP sockets
        let ipv4_socket = UdpSocket::bind("127.0.0.1:8080").await.unwrap();
        let ipv6_socket = UdpSocket::bind("[::1]:8080").await.unwrap();

        // Specify the target address for sending/receiving
        let target_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 8080);

        // Spawn tasks for continuous sending and receiving on both sockets
        let task_ipv4 = send_receive(ipv4_socket, target_addr, "Hello from IPv4!");
        let task_ipv6 = send_receive(ipv6_socket, target_addr, "Hello from IPv6!");

        // Wait for both tasks to complete (Ctrl+C to exit)
        tokio::try_join!(task_ipv4, task_ipv6).unwrap();
    });
}

*/
