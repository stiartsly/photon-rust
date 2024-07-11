use std::rc::Rc;
use std::cell::RefCell;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, LinkedList};

use log::{info, warn};
use tokio::io;
use tokio::runtime;
use tokio::net::UdpSocket;
use tokio::time::{sleep, interval_at, Duration};

use crate::{
    as_millis,
    constants,
    id::{self, Id},
    dht::DHT,
    error::Error,
    rpccall::RpcCall,
    scheduler::{self, Scheduler},
    msg::msg,
};

use crate::msg::msg::{Msg};

#[allow(dead_code)]
pub(crate) struct Server<> {
    nodeid: Id,
    started: SystemTime,

    reachable: bool,
    received_msgs: i32,
    msg_at_least_reachable_check: i32,
    last_reachable_check: SystemTime,

    calls: HashMap<i32, Rc<RefCell<RpcCall>>>,

    dht4: Option<Rc<RefCell<DHT>>>,
    queue4: Rc<RefCell<LinkedList<Rc<RefCell<dyn Msg>>>>>,

    scheduler:  Rc<RefCell<Scheduler>>,

}

// #[allow(dead_code)]
impl Server {
    pub fn new(nodeid: Id, ) -> Self {
        Self {
            nodeid,
            started: SystemTime::UNIX_EPOCH,

            reachable: false,
            received_msgs: 0,
            msg_at_least_reachable_check: 0,
            last_reachable_check: SystemTime::UNIX_EPOCH,

            calls: HashMap::new(),

            dht4: None,
            queue4: Rc::new(RefCell::new(LinkedList::new())),
            scheduler: Rc::new(RefCell::new(Scheduler::new())),
        }
    }

    pub(crate) fn scheduler(&self) -> Rc<RefCell<Scheduler>> {
        Rc::clone(&self.scheduler)
    }

    pub(crate) fn nodeid(&self) -> &Id {
        &self.nodeid
    }

    pub(crate) fn number_of_acitve_calls(&self) -> usize {
        self.calls.len()
    }

    pub(crate) fn start(&mut self, dht4: Rc<RefCell<DHT>>) -> Result<(), Error> {
        self.dht4 = Some(Rc::clone(&dht4));

        if let Some(dht4) = self.dht4.as_ref() {
            info!("Started RPC server on ipv4 address: {}", dht4.borrow().socket_addr());
        }

        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        self.dht4 = None;
    }

    pub(crate) fn is_reachable(&self) -> bool {
        self.reachable
    }

    pub(crate) fn update_reachability(&mut self) {
        // Avoid pinging too frequently if we're not receiving any response
        // (the connection might be dead)
        if self.received_msgs != self.msg_at_least_reachable_check {
            self.reachable = false;
            self.last_reachable_check = SystemTime::now();
            self.msg_at_least_reachable_check = self.received_msgs;
            return;
        }

        if as_millis!(self.last_reachable_check) >  constants::RPC_SERVER_REACHABILITY_TIMEOUT {
            self.reachable = false;
        }
    }

    //fn decrypt_into(&self, _: &Id, _: &[u8]) -> Result<Vec<u8>, Error> {
    //    unimplemented!()
    // }

    pub(crate) fn send_msg(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        msg.borrow_mut().set_id(self.nodeid());
        if let Some(call) = msg.borrow().associated_call() {
            call.borrow_mut().send();

            let call = Rc::clone(&call);
            self.scheduler.borrow_mut().add(move || {
               call.borrow_mut().check_timeout()
            }, 2000, 10);
        }

        self.queue4.borrow_mut().push_back(msg);
    }

    pub(crate) fn send_call(&mut self, call: Rc<RefCell<RpcCall>>) {
        let mut binding = call.borrow_mut();
        let hash = binding.hash();

        binding.set_responsed_fn(|_,_| {});
        binding.set_timeout_fn(|_call| {
            // self.on_timeout(_call);
        });
        drop(binding);

        self.calls.insert(hash, Rc::clone(&call));

        let req = match call.borrow().req() {
            Some(msg) => msg,
            None => return,
        };

        let mut binding = req.borrow_mut();
        binding.set_txid(hash);
        binding.with_associated_call(Rc::clone(&call));
        drop(binding);

        self.send_msg(req);
    }

    fn responsed(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        let txid = msg.borrow().txid();
        let call = match self.calls.remove(&txid) {
            Some(call) => call,
            None => return,
        };

        msg.borrow_mut().with_associated_call(Rc::clone(&call));
        call.borrow_mut().responsed(msg);
    }
}

pub(crate) fn run_loop(server: Rc<RefCell<Server>>,
    dht4: Rc<RefCell<DHT>>,
    quit: Arc<Mutex<bool>>
) -> io::Result<()>
{
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let buffer = Rc::new(RefCell::new(vec![0; 64*1024]));
    let mut running = true;

    rt.block_on(async move {
        let sock4 = UdpSocket::bind(dht4.borrow().socket_addr()).await?;
        let queue4 = Rc::clone(&server.borrow_mut().queue4);

        let mut interval = interval_at(
            server.borrow().scheduler.borrow().next_time(),
            Duration::from_secs(60*60)
        );
        while running {
            tokio::select! {
                data = read_socket(&sock4, Rc::clone(&buffer), move |_, buf| {
                   Ok(buf.to_vec())
                }) => {
                    if let Ok(Some(msg)) = data {
                        server.borrow_mut().responsed(Rc::clone(&msg));
                        dht4.borrow_mut().on_message(msg)
                    }
                }

                _ = write_socket(&sock4, Rc::clone(&dht4), Rc::clone(&queue4),  move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }) => {
                    //println!("Write data to ipv4 socket");
                }

                _ = interval.tick() => {
                    let scheduler = server.borrow().scheduler();
                    scheduler::run_jobs(scheduler);
                }
            }

            if *quit.lock().unwrap() {
                running = false;
            }
            if server.borrow().scheduler.borrow().is_updated() {
                interval.reset_at(server.borrow().scheduler.borrow().next_time());
            }
        }
        Ok(())
    })
}
async fn read_socket<F>(socket: &UdpSocket,
    buffer: Rc<RefCell<Vec<u8>>>,
    mut decrypt: F
) -> Result<Option<Rc<RefCell<dyn Msg>>>, io::Error>
    where F: FnMut(&Id, &mut [u8]) -> Result<Vec<u8>, Error>
{
    let mut buf = buffer.borrow_mut();
    let (len, from) = socket.recv_from(&mut buf).await?;
    let from_id = Id::from_bytes(&buf[0.. id::ID_BYTES]);

    let plain = match decrypt(&from_id, &mut buf[id::ID_BYTES .. len]) {
        Ok(plain) => plain,
        Err(err) => {
            warn!("Decrypt packet from {} error {}, discarded it", err, from);
            return Ok(None);
        }
    };

    let msg = match msg::deser(&plain) {
        Ok(msg) => msg,
        Err(err) => {
            warn!("Got a wrong packet from {} with {}", from, err);
            return Ok(None);
        }
    };

    msg.borrow_mut().set_id(&from_id);
    msg.borrow_mut().set_origin(&from);

    info!("Received message: {}/{} from {}:[size: {}] - {}", msg.borrow().method(), msg.borrow().kind(), from, len, msg.borrow());

    if msg.borrow().kind() != msg::Kind::Error && msg.borrow().txid() == 0 {
        warn!("Received a message with invalid transaction id, ignored it");
        return Ok(None);
    }

    // Just respond to incoming requests, no need to match them to pending requests
    if msg.borrow().kind() == msg::Kind::Request {
        return Ok(Some(msg));
    }

    Ok(Some(msg))
}

async fn write_socket<F>(socket: &UdpSocket,
    dht: Rc<RefCell<DHT>>,
    msg_queue: Rc<RefCell<LinkedList<Rc<RefCell<dyn Msg>>>>>, _: F) -> Result<(), io::Error>
where
    F: FnMut(&Id, &mut [u8]) -> Option<Vec<u8>>
{
    if msg_queue.borrow().is_empty() {
        sleep(Duration::MAX).await;
        return Ok(())
    }

    let msg = match msg_queue.borrow_mut().pop_front() {
        Some(msg) => msg,
        None => {
            sleep(Duration::from_millis(500)).await;
            return Ok(())
        }
    };

    if let Some(call) = msg.borrow().associated_call() {
        dht.borrow_mut().on_send(call.borrow().target_nodeid());
        call.borrow_mut().send();
        // self.scheduler.borrow_mut().add(move || {
        //    call.borrow_mut().check_timeout()
        // }, 2000, 10);
    }

    let serialized = msg::serialize(Rc::clone(&msg));
    let mut buf = Vec::new() as Vec<u8>;

    buf.extend_from_slice(msg.borrow().id().as_bytes());
    buf.extend_from_slice(&serialized);
    _ = socket.send_to(&buf, msg.borrow().remote_addr()).await?;

    Ok(())
}
