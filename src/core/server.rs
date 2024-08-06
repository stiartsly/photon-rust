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
    unwrap,
    as_millis,
    constants,
    id::{self, Id},
    dht::DHT,
    error::Error,
    rpccall::RpcCall,
    scheduler::{self, Scheduler},
    msg::msg::{self, Msg}
};

#[allow(dead_code)]
pub(crate) struct Server<> {
    nodeid: Rc<Id>,
    started: SystemTime,

    reachable: bool,
    received_msgs: i32,
    msg_at_least_reachable_check: i32,
    last_reachable_check: SystemTime,

    calls: Rc<RefCell<HashMap<i32, Rc<RefCell<RpcCall>>>>>,

    queue4: Option<Rc<RefCell<LinkedList<Rc<RefCell<dyn Msg>>>>>>,
    scheduler:  Rc<RefCell<Scheduler>>,

}

impl Server {
    pub fn new(input_nodeid: Rc<Id>) -> Self {
        Self {
            nodeid: input_nodeid,
            started: SystemTime::UNIX_EPOCH,

            reachable: false,
            received_msgs: 0,
            msg_at_least_reachable_check: 0,
            last_reachable_check: SystemTime::UNIX_EPOCH,

            calls: Rc::new(RefCell::new(HashMap::new())),
            queue4: Some(Rc::new(RefCell::new(LinkedList::new()))),

            scheduler: Rc::new(RefCell::new(Scheduler::new())),
        }
    }

    pub(crate) fn scheduler(&self) -> Rc<RefCell<Scheduler>> {
        self.scheduler.clone()
    }

    pub(crate) fn nodeid(&self) -> &Id {
        &self.nodeid
    }

    pub(crate) fn number_of_acitve_calls(&self) -> usize {
        self.calls.borrow().len()
    }

    pub(crate) fn start(&mut self) {}
    pub(crate) fn stop(&mut self) {}

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
        unwrap!(self.queue4).borrow_mut().push_back(msg);
    }

    pub(crate) fn send_call(&mut self, call: Rc<RefCell<RpcCall>>) {
        let txid = call.borrow().txid();

        let calls = self.calls.clone();
        calls.borrow_mut().insert(txid, call.clone());

        call.borrow_mut().set_responsed_fn(|_,_| {});
        call.borrow_mut().set_timeout_fn(move |_| {
            if let Some(c) = calls.borrow_mut().remove(&txid) {
                let dht = c.borrow().dht();
                dht.borrow_mut().on_timeout(c);
            };
        });

        if let Some(msg) = call.borrow().req() {
            msg.borrow_mut().set_txid(txid);
            msg.borrow_mut().with_associated_call(call.clone());
            self.send_msg(msg);
        };
    }

    fn responsed(&mut self, msg: Rc<RefCell<dyn Msg>>) {
        let txid = msg.borrow().txid();
        if let Some(call) = self.calls.borrow_mut().remove(&txid) {
            msg.borrow_mut().with_associated_call(call.clone());
            call.borrow_mut().responsed(msg);
        };
    }
}

pub(crate) fn run_loop(server: Rc<RefCell<Server>>,
    dht4: Option<Rc<RefCell<DHT>>>,
    dht6: Option<Rc<RefCell<DHT>>>,
    quit: Arc<Mutex<bool>>
) -> io::Result<()>
{
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let mut sock4 = None;
        let mut buf4 = None;
        let mut queue4 = None;

        if let Some(dht) = dht4.as_ref() {
            sock4 = Some(UdpSocket::bind(dht.borrow().socket_addr()).await?);
            buf4 = Some(Rc::new(RefCell::new(vec![0u8; 1024])));
            queue4 = server.borrow_mut().queue4.clone();
        }

        let mut sock6 = None;
        let mut buf6 = None;
        let mut queue6 = None;

        if let Some(dht) = dht6.as_ref() {
            sock6 = Some(UdpSocket::bind(dht.borrow().socket_addr()).await?);
            buf6 = Some(Rc::new(RefCell::new(vec![0u8; 1024])));
            queue6 = None;
        }

        let mut interval = interval_at(
            server.borrow().scheduler.borrow().next_timeout(),
            Duration::from_secs(60*60)
        );

        let mut running = true;
        while running {
            tokio::select! {
                res = read_socket(sock4.as_ref(), buf4.as_ref(), move |_, buf| {
                    Ok(buf.to_vec())
                }), if sock4.is_some() => {
                    match res {
                        Ok(data) => {
                            if let Some(msg) = data {
                                server.borrow_mut().responsed(msg.clone());
                                unwrap!(dht4).borrow_mut().on_message(msg)
                            }
                        },
                        Err(_) => {},
                    }
                }

                res = read_socket(sock6.as_ref(), buf6.as_ref(), move |_, buf| {
                    Ok(buf.to_vec())
                }), if sock6.is_some() => {
                    match res {
                        Ok(data) => {
                            if let Some(msg) = data {
                                server.borrow_mut().responsed(msg.clone());
                                unwrap!(dht6).borrow_mut().on_message(msg)
                            }
                        },
                        Err(_) => {},
                    }
                }

                res = write_socket(sock4.as_ref(), dht4.as_ref(), queue4.as_ref(), move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }), if sock4.is_some() => {
                    match res {
                        Ok(_) => {},
                        Err(_) => {},
                    }
                }

                res = write_socket(sock6.as_ref(), dht6.as_ref(), queue6.as_ref(), move |_, _| {
                    Some(Vec::new() as Vec<u8>)
                }), if sock6.is_some() => {
                    match res {
                        Ok(_) => {},
                        Err(_) => {},
                    }
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
                interval.reset_at(server.borrow().scheduler.borrow().next_timeout());
            }
        }

        Ok(())
    })
}
async fn read_socket<F>(socket: Option<&UdpSocket>,
    buffer: Option<&Rc<RefCell<Vec<u8>>>>,
    mut decrypt: F
) -> Result<Option<Rc<RefCell<dyn Msg>>>, io::Error>
where F: FnMut(&Id, &mut [u8]) -> Result<Vec<u8>, Error>
{
    let socket = match socket {
        Some(v) => v,
        None => return Ok(None),
    };

    let buf = match buffer {
        Some(v) => v,
        None => return Ok(None),
    };

    let mut buf = buf.borrow_mut();
    let (len, from) = socket.recv_from(&mut buf).await?;
    let from_id = Id::from_bytes(&buf[0.. id::ID_BYTES]);

    let plain = match decrypt(&from_id, &mut buf[id::ID_BYTES .. len]) {
        Ok(v) => v,
        Err(err) => {
            warn!("Decrypt packet from {} error {}, ignored it", err, from);
            return Ok(None);
        }
    };


    let msg = match msg::deser(&plain) {
        Ok(msg) => msg,
        Err(err) => {
            warn!("Got a wrong packet from {} with {}, ignored it", from, err);
            return Ok(None);
        }
    };

    msg.borrow_mut().set_id(&from_id);
    msg.borrow_mut().set_origin(&from);

    info!("Received message: {}/{} from {}:[size: {}] - {}",
        msg.borrow().method(),
        msg.borrow().kind(),
        from,
        len,
        msg.borrow());

    if msg.borrow().kind() != msg::Kind::Error && msg.borrow().txid() == 0 {
        warn!("Received a message with invalid txid, discarded it");
        return Ok(None);
    }

    // Just respond to incoming requests, no need to match them to pending requests
    if msg.borrow().kind() == msg::Kind::Request {
        return Ok(Some(msg));
    }

    Ok(Some(msg))
}

async fn write_socket<F>(socket: Option<&UdpSocket>,
    dht: Option<&Rc<RefCell<DHT>>>,
    queue: Option<&Rc<RefCell<LinkedList<Rc<RefCell<dyn Msg>>>>>>,
    _ : F
) -> Result<(), io::Error>
where F: FnMut(&Id, &mut [u8]) -> Option<Vec<u8>>
{
    let socket = match socket {
        Some(v) => v,
        None => return Ok(()),
    };

    let dht = match dht {
        Some(v) => v.clone(),
        None => return Ok(()),
    };

    let queue = match queue {
        Some(v) => v.clone(),
        None => return Ok(()),
    };

    let msg = match queue.borrow_mut().pop_front() {
        Some(v) => v,
        None => {
            sleep(Duration::from_millis(1000)).await;
            return Ok(())
        }
    };

    if let Some(call) = msg.borrow().associated_call() {
        let scheduler = dht.borrow().server().borrow().scheduler();

        dht.borrow_mut().on_send(call.borrow_mut().target_id());
        call.borrow_mut().send(scheduler);
    }

    let ser = msg::serialize(msg.clone());
    let mut buf = Vec::new() as Vec<u8>;

    buf.extend_from_slice(msg.borrow().id().as_bytes());
    buf.extend_from_slice(&ser);

    match socket.send_to(&buf, msg.borrow().remote_addr()).await {
        Ok(_) => {},
        Err(e) => warn!("Sending message failed {}", e),
    };

    Ok(())
}
