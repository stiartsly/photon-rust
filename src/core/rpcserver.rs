use tokio::io::{self, Interest};
use tokio::net::UdpSocket;
use tokio::runtime::{self};
//use crate::message::message;

use crate::dht::DHT;
use crate::rpccall::RPCCall;
use crate::msg::message::Message;

#[allow(dead_code)]
pub(crate) struct RpcServer {
    dht4: Option<Box<DHT>>,
    dht6: Option<Box<DHT>>,

    reachable: bool,
}

#[allow(dead_code)]
impl RpcServer {
    pub(crate) fn new() -> Self {
        RpcServer {
            dht4: None,
            dht6: None,
            reachable: false,
        }
    }

    pub(crate) fn enable_dht4(&mut self, dht4: Box<DHT>) -> &mut Self {
        self.dht4 = Some(dht4); self
    }

    pub(crate) fn enable_dht6(&mut self, dht6: Box<DHT>) -> &mut Self {
        self.dht6 = Some(dht6); self
    }

    pub(crate) fn is_reachable(&self) -> bool {
        self.reachable
    }

    fn run_loop(&self) -> io::Result<()> {
        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(async move {
            let socket = UdpSocket::bind("0.0.0.0:8080").await?;

            loop {
                let ready = socket.ready(Interest::READABLE | Interest::WRITABLE).await?;
                if ready.is_readable() {
                    let mut data = [0; 1024];
                    match socket.try_recv(&mut data[..]) {
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
                    match socket.try_send(b"hello world") {
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
        })
    }

    pub(crate) fn send_msg(&self, _: Box<dyn Message>) {
        unimplemented!()
    }

    pub(crate) fn send_call(&self, _: Box<RPCCall>) {
        unimplemented!()
    }

    pub(crate) fn send_err<'a>(&self, _: Box<dyn Message>, _: i32, _: &'a str) {
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
    rt.block_on(async {
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
