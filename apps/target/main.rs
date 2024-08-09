use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

use boson::{
    default_configuration,
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
    let path = get_storage_path(".target_data");

    let mut b = default_configuration::Builder::new();
    b.with_listening_port(39001);
    b.with_ipv4("192.168.12.121");
    // b.with_ipv4("172.20.10.2");
    b.with_storage_path(path.as_str());

    let cfg = b.build().unwrap();

    let mut runner = Node::new(cfg).unwrap();
    let _ = runner.start();

    thread::sleep(Duration::from_secs(100));
    runner.stop();
}
