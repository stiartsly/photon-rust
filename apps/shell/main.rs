use std::fs;
use std::env;

use boson::{
    default_configuration,
    runner::NodeRunner
};

fn get_storage_path(input: &str) -> String {
    let path = env::current_dir().unwrap().join(input);

    if !fs::metadata(&path).is_ok() {
        match fs::create_dir(&path) {
            Ok(_) => {},
            Err(e) => {
                panic!("Failed to create directory: {}", e);
            }
        }
    }
    path.display().to_string()
}

fn main() {
    let path = get_storage_path("shell");

    let mut b1 = default_configuration::Builder::new();
    b1.with_listening_port(32222);
    b1.with_ipv4("192.168.1.109");
    //b1.with_ipv4("192.168.237.121");
    b1.with_storage_path(path.as_str());
    let cfg1 = b1.build().unwrap();

    let mut runner = NodeRunner::new(cfg1).unwrap();
    println!("DHT node Id: {}", runner.id());

    let _ = runner.start();
    runner.stop();
}
