use std::thread;
use tokio::time::Duration;
use clap::Parser;

use boson::{
    Node,
    configuration as cfg
};

#[derive(Parser, Debug)]
#[command(about = "Boson Shell", long_about = None)]
struct Options {
    /// The configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    let opts = Options::parse();
    let cfg = cfg::Builder::new()
        .load(opts.config.as_ref().map_or("default.conf", |v|&v))
        .map_err(|e| eprintln!("{e}"))
        .unwrap()
        .build()
        .unwrap();

    #[cfg(feature = "inspect")] {
        cfg.dump();
    }

    let node = Node::new(cfg).unwrap();
    let _ = node.start();

    thread::sleep(Duration::from_secs(1));

    let target = "HZXXs9LTfNQjrDKvvexRhuMk8TTJhYCfrHwaj3jUzuhZ".try_into().unwrap();
    println!("Try to find node with id: {}", target);
    match node.find_node(&target, None).await {
        Ok(val) => match val.v4() {
            Some(node) => println!("Found node: {}", node.to_string()),
            None => println!("Not found !!!!"),
        },
        Err(e) => println!("error: {}", e),
    }

    thread::sleep(Duration::from_secs(10));
    node.stop();
}
