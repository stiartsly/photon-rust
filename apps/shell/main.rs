use std::env;
use std::fs;
use std::thread;
use std::time::Duration;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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

fn main() {
    let path = get_storage_path("shell");

    let mut b = default_configuration::Builder::new();
    b.with_listening_port(32222);
    b.with_ipv4("192.168.1.107");
    //b.with_ipv4("172.20.10.2");
    b.with_storage_path(path.as_str());

    let id = Id::try_from_base58("6o6LkHgLyD5sYyW9iN5LNRYnUoX29jiYauQ5cDjhCpWQ").unwrap();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 107)), 39001);
    let node = NodeInfo::new(&id, &addr);
    b.add_bootstrap(&node);

    let cfg = b.build().unwrap();

    let mut runner = Node::new(cfg).unwrap();
    let _ = runner.start();

    thread::sleep(Duration::from_secs(10));
    runner.stop();
}
