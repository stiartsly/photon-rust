use std::rc::Rc;
use std::cell::RefCell;
use std::io::Read;
use std::collections::LinkedList;
use std::thread::{self, JoinHandle};
use std::{fs, fs::File, io::Write};
use std::sync::{Arc, Mutex};
use log::{error, info};

use crate::{
    logger,
    signature,
    cryptobox::{self, Nonce},
    Id,
    Config,
    error::Error,
    NodeInfo,
    NodeStatus,
    Value,
    Peer,
    KeyPair,
    LookupOption,
    Compound,
    bootstrap_channel::BootstrapChannel,
    node_runner::{self, NodeRunner},
    future::{
        CmdFuture,
        FindNodeCmd,
        FindValueCmd,
        FindPeerCmd,
        StoreValueCmd,
        AnnouncePeerCmd,
        Command,
    }
};

pub struct Node {
    id: Id,
    cfg: Arc<Mutex<Box<dyn Config>>>, //config for this node.

    bootstrap_channel: Arc<Mutex<BootstrapChannel>>,
    command_channel: Arc<Mutex<LinkedList<Command>>>,

    signature_keypair: signature::KeyPair,
    encryption_keypair: cryptobox::KeyPair,

    option: LookupOption,
    status: NodeStatus,
    storage_path: String,

    thread: Option<JoinHandle<()>>, // working thread.
    quit: Arc<Mutex<bool>>, // notification handle for quit from working thread.
}

impl Node {
    pub fn new(cfg: Box<dyn Config>) -> Result<Self, Error> {
        logger::setup();

        // cfg(DEVELOPMENT)
        info!("DHT node running in development mode!!!");

        let mut path = cfg.storage_path().to_string();
        if path.is_empty() {
            path.push_str(".")
        }
        if !path.ends_with("/") {
            path.push_str("/");
        }

        let keypair = get_keypair(&path).map_err(|e| {
            error!("Acquire keypair from {} for DHT node error: {}", path, e);
            return e;
        }).ok().unwrap();

        let id = Id::from_signature_pubkey(keypair.public_key());
        let id_path = path.clone() + "id";
        store_nodeid(&id_path, &id).map_err(|e| {
            error!("Persisting node Id data error {}", e);
            return e
        }).ok().unwrap();

        info!("Current DHT node Id {}", id);

        Ok(Node {
            id,
            cfg: Arc::new(Mutex::new(cfg)),
            bootstrap_channel: Arc::new(Mutex::new(BootstrapChannel::new())),
            command_channel: Arc::new(Mutex::new(LinkedList::new())),
            signature_keypair: keypair.clone(),
            encryption_keypair: cryptobox::KeyPair::from_signature_keypair(&keypair),
            status: NodeStatus::Stopped,
            option: LookupOption::Conservative,
            storage_path: path,
            thread: None,
            quit: Arc::new(Mutex::new(false)),
        })
    }

    pub fn start(&mut self) {
        if self.status != NodeStatus::Stopped {
            return;
        }

        self.status = NodeStatus::Initializing;

        info!("DHT node <{}> is starting...", self.id);

        let path    = self.storage_path.clone();
        let keypair = self.signature_keypair.clone();
        let config  = self.cfg.clone();
        let channel = self.bootstrap_channel.clone();
        let cmds    = self.command_channel.clone();
        let quit    = self.quit.clone();

        self.bootstrap_many(config.lock().unwrap().bootstrap_nodes());
        self.thread = Some(thread::spawn(move || {
            let mut node = NodeRunner::new(keypair, path, config);
            node.set_bootstrap_channel(channel);
            node.set_command_channel(cmds);

            node_runner::run_loop(
                Rc::new(RefCell::new(node)), quit
            );
        }));

        self.status = NodeStatus::Running;
    }

    pub fn stop(&mut self) {
        if self.status == NodeStatus::Stopped {
            return;
        }

        info!("DHT node <{}> stopping...", self.id);

        // Check for abnormal termination in the spawned thread. If the thread is still
        // running, then notify it to abort.
        let mut quit = self.quit.lock().unwrap();
        if !*quit {
            *quit = true;
        }
        drop(quit);

        self.thread.take().unwrap().join().expect("Join thread error");
        self.thread = None;
        self.status = NodeStatus::Stopped;

        info!("DHT node {} stopped", self.id);
        logger::teardown();
    }

    pub fn is_running(&self) -> bool {
        self.status == NodeStatus::Running
    }

    pub fn bootstrap(&mut self, node: &NodeInfo) {
        self.bootstrap_channel.lock().unwrap().push(node);
    }

    pub fn bootstrap_many(&mut self, nodes: &[NodeInfo]) {
        self.bootstrap_channel.lock().unwrap().push_many(nodes);
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn is_self(&self, id: &Id) -> bool {
        &self.id == id
    }

    pub fn set_lookup_option(&mut self, option: LookupOption) {
        self.option = option;
    }

    pub fn lookup_option(&self) -> LookupOption {
        self.option.clone()
    }

    pub async fn find_node(&mut self, id: &Id, option: &LookupOption)
        -> Result<Compound<NodeInfo>, Error>
    {
        let arc = Arc::new(Mutex::new(FindNodeCmd::new(id, option)));
        let cmd = Command::FindNode(arc.clone());

        self.command_channel.lock()
            .unwrap()
            .push_back(cmd.clone());

        match CmdFuture::new(cmd).await {
            Ok(_) => arc.lock().unwrap().result(),
            Err(e) => Err(e)
        }
    }

    pub async fn find_node_simple(&mut self, id: &Id)
        -> Result<Compound<NodeInfo>, Error>
    {
        self.find_node(id, &self.option.clone()).await
    }

    pub async fn find_value(&self, id: &Id, option: &LookupOption)
        -> Result<Option<Value>, Error>
    {
        let arc = Arc::new(Mutex::new(FindValueCmd::new(id, option)));
        let cmd = Command::FindValue(arc.clone());

        self.command_channel.lock()
            .unwrap()
            .push_back(cmd.clone());

        match CmdFuture::new(cmd).await {
            Ok(_) => arc.lock().unwrap().result(),
            Err(e) => Err(e)
        }
    }

    pub async fn find_value_simple(&self, value_id: &Id)
        -> Result<Option<Value>, Error>
    {
        self.find_value(value_id, &self.option).await
    }

    pub async fn find_peer(&self, id: &Id, expected_seq: i32, option: &LookupOption)
        -> Result<Vec<Peer>, Error>
    {
        let arc = Arc::new(Mutex::new(FindPeerCmd::new(id, expected_seq, option)));
        let cmd = Command::FindPeer(arc.clone());

        self.command_channel.lock()
            .unwrap()
            .push_back(cmd.clone());

        match CmdFuture::new(cmd).await {
            Ok(_) => arc.lock().unwrap().result(),
            Err(e) => Err(e)
        }
    }

    pub async fn find_peer_simple(&self, peer_id: &Id, expected_seq: i32)
        -> Result<Vec<Peer>, Error>
    {
        self.find_peer(peer_id, expected_seq, &self.option).await
    }

    pub async fn store_value(&mut self, value: &Value, _: bool)
        -> Result<(), Error>
    {
        let arc = Arc::new(Mutex::new(StoreValueCmd::new(value)));
        let cmd = Command::StoreValue(arc.clone());

        self.command_channel.lock()
            .unwrap()
            .push_back(cmd.clone());

        match CmdFuture::new(cmd).await {
            Ok(_) => arc.lock().unwrap().result(),
            Err(e) => Err(e)
        }
    }

    pub async fn store_value_simple(&mut self, value: &Value, _: bool)
        -> Result<(), Error>
    {
        self.store_value(value, false).await
    }

    pub async fn announce_peer(&mut self, peer: &Peer, _: bool)
        -> Result<(), Error>
    {
        let arc = Arc::new(Mutex::new(AnnouncePeerCmd::new(peer)));
        let cmd = Command::AnnouncePeer(arc.clone());

        self.command_channel.lock()
            .unwrap()
            .push_back(cmd.clone());

        match CmdFuture::new(cmd.clone()).await {
            Ok(_) => arc.lock().unwrap().result(),
            Err(e) => Err(e)
        }
    }

    pub async fn announce_peer_simple(&mut self, peer: &Peer) -> Result<(), Error> {
        self.announce_peer(peer, false).await
    }

    pub fn encrypt_into(&self, recipient: &Id, plain: &[u8]) -> Result<Vec<u8>, Error> {
        let nonce = Nonce::from(
            Id::distance(&self.id, &recipient).as_bytes()
        );

        cryptobox::encrypt_into(
            plain,
            &nonce,
            &recipient.to_encryption_pubkey(),
            self.encryption_keypair.private_key()
        )
    }

    pub fn decrypt_into(&self, sender: &Id, cipher: &[u8]) -> Result<Vec<u8>, Error> {
        let nonce = Nonce::from(
            Id::distance(&self.id, &sender).as_bytes()
        );

        cryptobox::decrypt_into(
            cipher,
            &nonce,
            &sender.to_encryption_pubkey(),
            self.encryption_keypair.private_key()
        )
    }

    pub fn encrypt(&self, recipient: &Id, plain: &[u8], cipher: &mut [u8]) -> Result<(), Error> {
        let nonce = Nonce::from(
            Id::distance(&self.id, &recipient).as_bytes()
        );

        cryptobox::encrypt(
            cipher,
            plain,
            &nonce,
            &recipient.to_encryption_pubkey(),
            self.encryption_keypair.private_key()
        )
    }

    pub fn decrypt(&self, sender: &Id, cipher: &[u8], plain: &mut [u8]) -> Result<(), Error> {
        let nonce = Nonce::from(
            Id::distance(&self.id, &sender).as_bytes()
        );

        cryptobox::decrypt(
            plain,
            cipher,
            &nonce,
            &sender.to_encryption_pubkey(),
            self.encryption_keypair.private_key()
        )
    }

    pub fn sign_into(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(
            self.signature_keypair
                .private_key()
                .sign_into(data)
        )
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), Error> {
        self.signature_keypair
            .public_key()
            .verify(data, signature);

        Ok(())
    }
}

fn get_keypair(path: &str) -> Result<KeyPair, Error> {
    check_persistence(path).map_err(|e| {
        return Error::State(format!("Checking persistence error: {}", e));
    }).ok().unwrap();

    let keypath = path.to_string() + "key";
    let keypair;

    match fs::metadata(&keypath) {
        Ok(metadata) => {
            // Loading key from persistence.
            if metadata.is_dir() {
                return Err(Error::State(format!("Bad file path {} for key storage.", keypath)));
            };
            keypair = load_key(&keypath)
                .map_err(|e| return e)
                .ok()
                .unwrap();
        },
        Err(_) => {
            // otherwise, generate a fresh keypair
            keypair = KeyPair::random();
            store_key(&keypath, &keypair)
                .map_err(|e|return e)
                .ok()
                .unwrap();
        }
    };

    Ok(keypair)
}

fn load_key(path: &str) -> Result<KeyPair, Error> {
    let mut fp = match File::open(path) {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(
            e, "Openning key file error".to_string())),
    };

    let mut buf = Vec::new();
    if let Err(e) = fp.read_to_end(&mut buf) {
        return Err(Error::Io(e, "Reading key error".to_string()));
    };

    if buf.len() != signature::PrivateKey::BYTES {
        return Err(Error::State(format!(
            "Incorrect key size {}", buf.len())));
    }

    Ok(KeyPair::from_private_key_bytes(&buf))
}

fn store_key(path: &str, keypair: &KeyPair) -> Result<(), Error> {
    let mut fp = match File::create(path) {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(
            e, format!("Creating key file error"))),
    };

    if let Err(e) = fp.write_all(keypair.private_key().as_bytes()) {
        return Err(Error::Io(e, format!("Writing key error.")))
    }

    Ok(())
}

fn store_nodeid(path: &str, id: &Id) -> Result<(), Error> {
    let mut fp = match File::create(path) {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(
            e, format!("Creating Id file error"))),
    };

    if let Err(e) = fp.write_all(id.as_bytes()) {
        return Err(Error::Io(e, format!("Writing ID failed")));
    }

    Ok(())
}

fn check_persistence(_: &str) -> Result<(), Error> {
    // TODO:
    Ok(())
}
