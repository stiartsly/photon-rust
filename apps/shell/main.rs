use std::env;
use std::fs;
use std::thread;
use tokio::time::Duration;

use std::net::SocketAddr;

use boson::{
    default_configuration,
    id::Id,
    NodeInfo,
    node::Node
};

fn get_storage_path(input: &str) -> String {
    let path = env::current_dir().unwrap().join(input);

    if !fs::metadata(&path).is_ok() {
        match fs::create_dir(&path) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to create directory: {}", e);
            }
        }
    }
    path.display().to_string()
}

#[tokio::main]
async fn main() {
    let path = get_storage_path(".shell_data");

    let mut b = default_configuration::Builder::new();
    b.with_listening_port(32222);
    b.with_ipv4("192.168.1.107");
    b.with_storage_path(path.as_str());

    let id = Id::try_from_base58("HwrxvgqmY2UCweXH7bV64wNZB8thpgweUTX47N17NJA").unwrap();
    let addr = "192.168.1.107:39001".parse::<SocketAddr>().ok().unwrap();
    let node = NodeInfo::new(&id, &addr);
    b.add_bootstrap(&node);

    let id = Id::try_from_base58("HZXXs9LTfNQjrDKvvexRhuMk8TTJhYCfrHwaj3jUzuhZ").unwrap();
    let addr = "155.138.245.211:39001".parse::<SocketAddr>().ok().unwrap();
    let node = NodeInfo::new(&id, &addr);
    b.add_bootstrap(&node);

    let cfg = b.build().unwrap();
    println!("bootstrap nodes: {}\n", cfg.bootstrap_nodes().len());

    let mut node = Node::new(cfg).unwrap();
    let _ = node.start();

    thread::sleep(Duration::from_secs(1));
    match node.find_node_simple(&id).await {
        Ok(_) => panic!("Got response!!!!!!!!!!!"),
        Err(e) => println!("error: {}", e),
    }

    thread::sleep(Duration::from_secs(10));
    node.stop();
}
